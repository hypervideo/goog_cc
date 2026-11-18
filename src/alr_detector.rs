/*
 *  Copyright (c) 2016 The WebRTC project authors. All Rights Reserved.
 *
 *  Use of this source code is governed by a BSD-style license
 *  that can be found in the LICENSE file in the root of the source
 *  tree. An additional intellectual property rights grant can be found
 *  in the file PATENTS.  All contributing project authors may
 *  be found in the AUTHORS file in the root of the source tree.
 */

use std::sync::Arc;

use crate::{
    api::{
        rtc_event_log::{RtcEvent, RtcEventAlrState, RtcEventLog, RtcEventLogNull},
        units::{DataRate, DataSize, TimeDelta, Timestamp},
    },
    experiments::AlrExperimentSettings,
    pacing::IntervalBudget,
};

#[derive(Clone, Debug)]
pub struct AlrDetectorConfig {
    // Sent traffic ratio as a function of network capacity used to determine
    // application-limited region. ALR region start when bandwidth usage drops
    // below AlrStartUsageRatio and ends when it raises above
    // AlrEndUsageRatio. NOTE: This is intentionally conservative at the moment
    // until BW adjustments of application limited region is fine tuned.
    pub bandwidth_usage_ratio: f64,
    pub start_budget_level_ratio: f64,
    pub stop_budget_level_ratio: f64,
}

impl Default for AlrDetectorConfig {
    fn default() -> Self {
        // The ALR experiment seems hard-coded on.
        // TODO Technically, WebRTC-AlrDetectorParameters could override these values.
        let settings = AlrExperimentSettings::default();
        Self {
            bandwidth_usage_ratio: settings.alr_bandwidth_usage_percent as f64 / 100.0,
            start_budget_level_ratio: settings.alr_start_budget_level_percent as f64 / 100.0,
            stop_budget_level_ratio: settings.alr_stop_budget_level_percent as f64 / 100.0,
        }
    }
}

// Application limited region detector is a class that utilizes signals of
// elapsed time and bytes sent to estimate whether network traffic is
// currently limited by the application's ability to generate traffic.
//
// AlrDetector provides a signal that can be utilized to adjust
// estimate bandwidth.
// Note: This class is not thread-safe.
pub struct AlrDetector {
    event_log: Arc<dyn RtcEventLog>,
    conf: AlrDetectorConfig,

    last_send_time: Option<Timestamp>,

    alr_budget: IntervalBudget,
    alr_started_time: Option<Timestamp>,
}

impl Default for AlrDetector {
    fn default() -> Self {
        Self::new(AlrDetectorConfig::default(), Arc::new(RtcEventLogNull))
    }
}

impl AlrDetector {
    pub fn new(conf: AlrDetectorConfig, event_log: Arc<dyn RtcEventLog>) -> Self {
        Self {
            event_log,
            conf,
            last_send_time: None,
            alr_budget: IntervalBudget::new(0, true),
            alr_started_time: None,
        }
    }

    pub fn on_bytes_sent(&mut self, bytes_sent: DataSize, send_time: Timestamp) {
        let last_send_time = match self.last_send_time {
            Some(v) => v,
            None => {
                // Since the duration for sending the bytes is unknwon, return without
                // updating alr state.
                self.last_send_time = Some(send_time);
                return;
            }
        };

        let delta_time: TimeDelta = send_time - last_send_time;
        self.last_send_time = Some(send_time);

        self.alr_budget.use_budget(bytes_sent.bytes() as usize);
        self.alr_budget.increase_budget(delta_time.ms());
        let mut state_changed = false;
        if self.alr_budget.budget_ratio() > self.conf.start_budget_level_ratio
            && self.alr_started_time.is_none()
        {
            // libwebrtc stamps the start with the current time of its
            // environment's clock. Callers pass `send_time` on that clock, at
            // the time of the call.
            self.alr_started_time = Some(send_time);
            state_changed = true;
        } else if self.alr_budget.budget_ratio() < self.conf.stop_budget_level_ratio
            && self.alr_started_time.is_some()
        {
            state_changed = true;
            self.alr_started_time = None;
        }
        if state_changed {
            self.event_log.log(RtcEvent::AlrState(RtcEventAlrState {
                in_alr: self.alr_started_time.is_some(),
            }));
        }
    }

    // Set current estimated bandwidth.
    pub fn set_estimated_bitrate(&mut self, bitrate: DataRate) {
        debug_assert!(bitrate > DataRate::zero());
        if bitrate <= DataRate::zero() {
            return;
        }
        self.alr_budget
            .set_target_rate_kbps((bitrate * self.conf.bandwidth_usage_ratio).kbps());
    }

    // Returns time when the current application-limited region started or empty
    // result if the sender is currently not application-limited.
    pub fn get_application_limited_region_start_time(&self) -> Option<Timestamp> {
        self.alr_started_time
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::api::rtc_event_log::test::RecordingRtcEventLog;

    const ESTIMATED_BITRATE: DataRate = DataRate::from_bits_per_sec(300_000);

    pub struct SimulateOutgoingTrafficIn<'a> {
        alr_detector: &'a mut AlrDetector,
        clock: &'a mut Timestamp,
        interval: Option<TimeDelta>,
        usage_percentage: Option<i64>,
    }

    impl<'a> SimulateOutgoingTrafficIn<'a> {
        pub fn new(alr_detector: &'a mut AlrDetector, clock: &'a mut Timestamp) -> Self {
            Self {
                alr_detector,
                clock,
                interval: None,
                usage_percentage: None,
            }
        }

        pub fn for_time(mut self, time: TimeDelta) -> Self {
            self.interval.replace(time);
            self.produce_traffic();
            self
        }

        pub fn at_percent_of_estimated_bitrate(mut self, usage_percentage: i64) -> Self {
            self.usage_percentage.replace(usage_percentage);
            self.produce_traffic();
            self
        }

        fn produce_traffic(&mut self) {
            let (Some(interval), Some(usage_percentage)) = (self.interval, self.usage_percentage)
            else {
                return;
            };
            const TIME_STEP: TimeDelta = TimeDelta::from_millis(10);
            let mut t = TimeDelta::zero();
            while t < interval {
                *self.clock += TIME_STEP;
                self.alr_detector.on_bytes_sent(
                    ESTIMATED_BITRATE * usage_percentage * TIME_STEP / 100,
                    *self.clock,
                );
                t += TIME_STEP;
            }
            // As of now all tests use interval that is a multiple of 10ms.
            debug_assert_eq!(interval.ms() % TIME_STEP.ms(), 0);
        }
    }

    #[test]
    fn alr_detection() {
        let mut clock = Timestamp::from_seconds(1);
        let mut alr_detector = AlrDetector::default();
        alr_detector.set_estimated_bitrate(ESTIMATED_BITRATE);

        // Start in non-ALR state.
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_none());

        // Stay in non-ALR state when usage is close to 100%.
        SimulateOutgoingTrafficIn::new(&mut alr_detector, &mut clock)
            .for_time(TimeDelta::from_seconds(1))
            .at_percent_of_estimated_bitrate(90);
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_none());

        // Verify that we ALR starts when bitrate drops below 20%.
        SimulateOutgoingTrafficIn::new(&mut alr_detector, &mut clock)
            .for_time(TimeDelta::from_millis(1_500))
            .at_percent_of_estimated_bitrate(20);
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_some());

        // Verify that ALR ends when usage is above 65%.
        SimulateOutgoingTrafficIn::new(&mut alr_detector, &mut clock)
            .for_time(TimeDelta::from_seconds(4))
            .at_percent_of_estimated_bitrate(100);
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_none());
    }

    /// The detector logs each entry into and exit from the application limited
    /// region, as libwebrtc's `OnBytesSent` does
    /// ([`alr_detector.cc#84`](https://webrtc.googlesource.com/src/+/3075edeba1/modules/congestion_controller/goog_cc/alr_detector.cc#84)).
    ///
    /// The traffic is that of [`alr_detection`].
    #[test]
    fn logs_alr_state_changes() {
        let event_log = Arc::new(RecordingRtcEventLog::default());
        let mut clock = Timestamp::from_seconds(1);
        let mut alr_detector = AlrDetector::new(AlrDetectorConfig::default(), event_log.clone());
        alr_detector.set_estimated_bitrate(ESTIMATED_BITRATE);

        SimulateOutgoingTrafficIn::new(&mut alr_detector, &mut clock)
            .for_time(TimeDelta::from_seconds(1))
            .at_percent_of_estimated_bitrate(90);
        assert_eq!(event_log.take(), []);

        SimulateOutgoingTrafficIn::new(&mut alr_detector, &mut clock)
            .for_time(TimeDelta::from_millis(1_500))
            .at_percent_of_estimated_bitrate(20);
        assert_eq!(
            event_log.take(),
            [RtcEvent::AlrState(RtcEventAlrState { in_alr: true })]
        );

        SimulateOutgoingTrafficIn::new(&mut alr_detector, &mut clock)
            .for_time(TimeDelta::from_seconds(4))
            .at_percent_of_estimated_bitrate(100);
        assert_eq!(
            event_log.take(),
            [RtcEvent::AlrState(RtcEventAlrState { in_alr: false })]
        );
    }

    #[test]
    fn short_spike() {
        let mut clock = Timestamp::from_seconds(1);
        let mut alr_detector = AlrDetector::default();
        alr_detector.set_estimated_bitrate(ESTIMATED_BITRATE);
        // Start in non-ALR state.
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_none());

        // Verify that we ALR starts when bitrate drops below 20%.
        SimulateOutgoingTrafficIn::new(&mut alr_detector, &mut clock)
            .for_time(TimeDelta::from_seconds(1))
            .at_percent_of_estimated_bitrate(20);
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_some());

        // Verify that we stay in ALR region even after a short bitrate spike.
        SimulateOutgoingTrafficIn::new(&mut alr_detector, &mut clock)
            .for_time(TimeDelta::from_millis(100))
            .at_percent_of_estimated_bitrate(150);
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_some());

        // ALR ends when usage is above 65%.
        SimulateOutgoingTrafficIn::new(&mut alr_detector, &mut clock)
            .for_time(TimeDelta::from_seconds(3))
            .at_percent_of_estimated_bitrate(100);
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_none());
    }

    #[test]
    fn bandwidth_estimate_changes() {
        let mut clock = Timestamp::from_seconds(1);
        let mut alr_detector = AlrDetector::default();
        alr_detector.set_estimated_bitrate(ESTIMATED_BITRATE);

        // Start in non-ALR state.
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_none());

        // ALR starts when bitrate drops below 20%.
        SimulateOutgoingTrafficIn::new(&mut alr_detector, &mut clock)
            .for_time(TimeDelta::from_seconds(1))
            .at_percent_of_estimated_bitrate(20);
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_some());

        // When bandwidth estimate drops the detector should stay in ALR mode and quit
        // it shortly afterwards as the sender continues sending the same amount of
        // traffic. This is necessary to ensure that ProbeController can still react
        // to the BWE drop by initiating a new probe.
        alr_detector.set_estimated_bitrate(ESTIMATED_BITRATE / 5);
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_some());
        SimulateOutgoingTrafficIn::new(&mut alr_detector, &mut clock)
            .for_time(TimeDelta::from_seconds(1))
            .at_percent_of_estimated_bitrate(50);
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_none());
    }

    #[test]
    fn parse_alr_specific_field_trial() {
        let config = AlrDetectorConfig {
            bandwidth_usage_ratio: 0.90,
            start_budget_level_ratio: 0.0,
            stop_budget_level_ratio: -0.10,
        };
        let mut clock = Timestamp::from_seconds(1);
        let mut alr_detector = AlrDetector::new(config, Arc::new(RtcEventLogNull));
        alr_detector.set_estimated_bitrate(ESTIMATED_BITRATE);

        // Start in non-ALR state.
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_none());

        // ALR does not start at 100% utilization.
        SimulateOutgoingTrafficIn::new(&mut alr_detector, &mut clock)
            .for_time(TimeDelta::from_seconds(1))
            .at_percent_of_estimated_bitrate(100);
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_none());

        // ALR does start at 85% utilization.
        // Overused 10% above so it should take about 2s to reach a budget level of
        // 0%.
        SimulateOutgoingTrafficIn::new(&mut alr_detector, &mut clock)
            .for_time(TimeDelta::from_millis(2_100))
            .at_percent_of_estimated_bitrate(85);
        assert!(alr_detector
            .get_application_limited_region_start_time()
            .is_some());
    }
}
