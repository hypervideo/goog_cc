/*
 *  Copyright (c) 2017 The WebRTC project authors. All Rights Reserved.
 *
 *  Use of this source code is governed by a BSD-style license
 *  that can be found in the LICENSE file in the root of the source
 *  tree. An additional intellectual property rights grant can be found
 *  in the file PATENTS.  All contributing project authors may
 *  be found in the AUTHORS file in the root of the source tree.
 */

//! The RTC event log and the events that goog_cc writes to it.
//!
//! libwebrtc's [`api/rtc_event_log`](https://webrtc.googlesource.com/src/+/3075edeba1/api/rtc_event_log/)
//! and the goog_cc events of
//! [`logging/rtc_event_log/events`](https://webrtc.googlesource.com/src/+/3075edeba1/logging/rtc_event_log/events/).

use std::fmt;

use super::{transport::BandwidthUsage, units::DataRate};

/// A log that the controller's components write [`RtcEvent`]s to.
///
/// libwebrtc's `RtcEventLog`
/// ([`rtc_event_log.h`](https://webrtc.googlesource.com/src/+/3075edeba1/api/rtc_event_log/rtc_event_log.h)),
/// which [`NetworkControllerConfig::event_log`] gives the controller. The port
/// keeps only `Log`: libwebrtc's `StartLogging` and `StopLogging` manage the
/// output of its log implementation.
///
/// The components call [`Self::log`] at libwebrtc's call sites, within the call
/// to the controller that caused the event. libwebrtc's log stamps each event
/// with the time of its clock. The port's events carry no time, so a log that
/// needs one takes it from the controller call.
///
/// [`NetworkControllerConfig::event_log`]: super::network_control::NetworkControllerConfig::event_log
pub trait RtcEventLog: Send + Sync {
    /// Logs `event`.
    fn log(&self, event: RtcEvent);
}

impl fmt::Debug for dyn RtcEventLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RtcEventLog")
    }
}

/// An [`RtcEventLog`] that drops every event, libwebrtc's `RtcEventLogNull`.
#[derive(Debug, Clone, Copy, Default)]
pub struct RtcEventLogNull;

impl RtcEventLog for RtcEventLogNull {
    fn log(&self, _event: RtcEvent) {}
}

/// An event that goog_cc logs, one variant for each of libwebrtc's `RtcEvent`
/// types that goog_cc creates
/// ([`rtc_event.h`](https://webrtc.googlesource.com/src/+/3075edeba1/api/rtc_event_log/rtc_event.h)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RtcEvent {
    AlrState(RtcEventAlrState),
    BweUpdateDelayBased(RtcEventBweUpdateDelayBased),
    BweUpdateLossBased(RtcEventBweUpdateLossBased),
    ProbeClusterCreated(RtcEventProbeClusterCreated),
    ProbeResultFailure(RtcEventProbeResultFailure),
    ProbeResultSuccess(RtcEventProbeResultSuccess),
    RemoteEstimate(RtcEventRemoteEstimate),
}

/// The ALR detector found the sender entering or leaving the application
/// limited region
/// ([`rtc_event_alr_state.h`](https://webrtc.googlesource.com/src/+/3075edeba1/logging/rtc_event_log/events/rtc_event_alr_state.h)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcEventAlrState {
    pub in_alr: bool,
}

/// The delay-based estimate or the state of its overuse detector changed
/// ([`rtc_event_bwe_update_delay_based.h`](https://webrtc.googlesource.com/src/+/3075edeba1/logging/rtc_event_log/events/rtc_event_bwe_update_delay_based.h)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcEventBweUpdateDelayBased {
    pub bitrate_bps: i32,
    pub detector_state: BandwidthUsage,
}

/// The send-side estimate set its target, which the delay-based estimate
/// caps
/// ([`rtc_event_bwe_update_loss_based.h`](https://webrtc.googlesource.com/src/+/3075edeba1/logging/rtc_event_log/events/rtc_event_bwe_update_loss_based.h)).
///
/// It is logged when the target or the fraction of lost packets changes, and on
/// any other update of the target more than 5s after the last event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcEventBweUpdateLossBased {
    pub bitrate_bps: i32,
    /// The fraction of packets lost, in 1/256ths.
    pub fraction_loss: u8,
    /// The packets expected since the last update of the fraction of lost
    /// packets.
    pub total_packets: i32,
}

/// The probe controller created a probe cluster
/// ([`rtc_event_probe_cluster_created.h`](https://webrtc.googlesource.com/src/+/3075edeba1/logging/rtc_event_log/events/rtc_event_probe_cluster_created.h)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcEventProbeClusterCreated {
    pub id: i32,
    pub bitrate_bps: i32,
    pub min_probes: u32,
    pub min_bytes: u32,
}

/// The probe bitrate estimator rejected the feedback of a probe cluster
/// ([`rtc_event_probe_result_failure.h`](https://webrtc.googlesource.com/src/+/3075edeba1/logging/rtc_event_log/events/rtc_event_probe_result_failure.h)).
///
/// The estimator evaluates a cluster on each probe packet's feedback once it
/// has enough of them, so it can log several results for one cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcEventProbeResultFailure {
    pub id: i32,
    pub failure_reason: ProbeFailureReason,
}

/// Why the probe bitrate estimator rejected the feedback of a probe cluster.
///
/// libwebrtc's `ProbeFailureReason` also has `kTimeout`, which goog_cc does
/// not log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeFailureReason {
    InvalidSendReceiveInterval,
    InvalidSendReceiveRatio,
}

/// The probe bitrate estimator derived an estimate from the feedback of a probe
/// cluster
/// ([`rtc_event_probe_result_success.h`](https://webrtc.googlesource.com/src/+/3075edeba1/logging/rtc_event_log/events/rtc_event_probe_result_success.h)).
///
/// Like [`RtcEventProbeResultFailure`], it can be logged several times for one
/// cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcEventProbeResultSuccess {
    pub id: i32,
    pub bitrate_bps: i32,
}

/// The controller got a new network state estimate
/// ([`rtc_event_remote_estimate.h`](https://webrtc.googlesource.com/src/+/3075edeba1/logging/rtc_event_log/events/rtc_event_remote_estimate.h)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcEventRemoteEstimate {
    pub link_capacity_lower: DataRate,
    pub link_capacity_upper: DataRate,
}

#[cfg(test)]
pub(crate) mod test {
    use std::sync::Mutex;

    use super::*;

    /// An [`RtcEventLog`] that keeps the events it is given.
    #[derive(Debug, Default)]
    pub(crate) struct RecordingRtcEventLog(Mutex<Vec<RtcEvent>>);

    impl RecordingRtcEventLog {
        /// Takes the events logged since the last call.
        pub(crate) fn take(&self) -> Vec<RtcEvent> {
            std::mem::take(&mut self.0.lock().unwrap())
        }
    }

    impl RtcEventLog for RecordingRtcEventLog {
        fn log(&self, event: RtcEvent) {
            self.0.lock().unwrap().push(event);
        }
    }
}
