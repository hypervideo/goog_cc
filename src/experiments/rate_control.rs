/*
 *  Copyright (c) 2019 The WebRTC project authors. All Rights Reserved.
 *
 *  Use of this source code is governed by a BSD-style license
 *  that can be found in the LICENSE file in the root of the source
 *  tree. An additional intellectual property rights grant can be found
 *  in the file PATENTS.  All contributing project authors may
 *  be found in the AUTHORS file in the root of the source tree.
 */

use crate::{api::units::DataSize, experiments::FieldTrials};

/// WebRTC-CongestionWindow
#[derive(Clone, Debug)]
pub struct CongestionWindowConfig {
    /// The queuing time that the congestion window allows on top of the RTT.
    ///
    /// The window is the data that the target rate sends in the RTT plus this
    /// time. Without a queue size, the controller keeps no congestion window,
    /// and zero gives one of the RTT alone.
    pub queue_size_ms: Option<i64>, // QueueSize
    /// The lowest target bitrate that congestion window pushback reduces the
    /// encoder target to.
    ///
    /// Pushback leaves a target that is already lower as it is. Zero gives
    /// pushback without a floor. Without a value, the controller does no
    /// pushback and passes the congestion window to the pacer instead.
    pub min_bitrate_bps: Option<i64>, // MinBitrate
    pub initial_data_window: Option<DataSize>, // InitWin
    pub drop_frame_only: bool,                 // DropFrame
}

impl Default for CongestionWindowConfig {
    // QueueSize:350,MinBitrate:30000,DropFrame:true
    fn default() -> Self {
        Self {
            queue_size_ms: Some(RateControlSettings::DEFAULT_ACCEPTED_QUEUE_MS),
            min_bitrate_bps: Some(RateControlSettings::DEFAULT_MIN_PUSHBACK_TARGET_BITRATE_BPS),
            initial_data_window: None,
            drop_frame_only: true,
        }
    }
}

/// WebRTC-VideoRateControl
#[derive(Clone, Debug)]
pub struct VideoRateControlConfig {
    pub pacing_factor: Option<f64>,
    pub alr_probing: bool,
    pub vp8_qp_max: Option<i64>,
    pub vp8_min_pixels: Option<i64>,
    pub trust_vp8: bool,             // = true
    pub trust_vp9: bool,             // = true;
    pub bitrate_adjuster: bool,      // = true;
    pub adjuster_use_headroom: bool, // = true;
    pub vp8_s0_boost: bool,
    pub vp8_base_heavy_tl3_alloc: bool,
}

impl Default for VideoRateControlConfig {
    fn default() -> Self {
        Self {
            pacing_factor: None,
            alr_probing: false,
            vp8_qp_max: None,
            vp8_min_pixels: None,
            trust_vp8: true,
            trust_vp9: true,
            bitrate_adjuster: true,
            adjuster_use_headroom: true,
            vp8_s0_boost: false,
            vp8_base_heavy_tl3_alloc: false,
        }
    }
}

#[derive(Default)]
pub struct RateControlSettings {
    congestion_window_config: CongestionWindowConfig,
    video_config: VideoRateControlConfig,
}

impl RateControlSettings {
    const DEFAULT_ACCEPTED_QUEUE_MS: i64 = 350;
    const DEFAULT_MIN_PUSHBACK_TARGET_BITRATE_BPS: i64 = 30000;

    // const char kCongestionWindowDefaultFieldTrialString[] =
    //"QueueSize:350,MinBitrate:30000,DropFrame:true";
    //const char kUseBaseHeavyVp8Tl3RateAllocationFieldTrialName[] =
    //   "WebRTC-UseBaseHeavyVP8TL3RateAllocation";

    pub fn new(field_trials: &FieldTrials) -> Self {
        let congestion_window_config = field_trials.congestion_window.clone();
        let mut video_config = field_trials.video_rate_control.clone();
        video_config.vp8_base_heavy_tl3_alloc =
            video_config.vp8_base_heavy_tl3_alloc || field_trials.vp8_base_heavy_tl3_alloc;
        Self {
            congestion_window_config,
            video_config,
        }
    }

    /// When CongestionWindowPushback is enabled, the pacer is oblivious to
    /// the congestion window. The relation between outstanding data and
    /// the congestion window affects encoder allocations directly.
    pub fn use_congestion_window(&self) -> bool {
        self.congestion_window_config.queue_size_ms.is_some()
    }

    pub fn get_congestion_window_additional_time_ms(&self) -> i64 {
        self.congestion_window_config
            .queue_size_ms
            .unwrap_or(Self::DEFAULT_ACCEPTED_QUEUE_MS)
    }

    pub fn use_congestion_window_pushback(&self) -> bool {
        self.congestion_window_config.queue_size_ms.is_some()
            && self.congestion_window_config.min_bitrate_bps.is_some()
    }

    pub fn use_congestion_window_drop_frame_only(&self) -> bool {
        self.congestion_window_config.drop_frame_only
    }

    pub fn congestion_window_min_pushback_target_bitrate_bps(&self) -> u32 {
        self.congestion_window_config
            .min_bitrate_bps
            .unwrap_or(Self::DEFAULT_MIN_PUSHBACK_TARGET_BITRATE_BPS) as _
    }

    pub fn congestion_window_initial_data_window(&self) -> Option<DataSize> {
        self.congestion_window_config.initial_data_window
    }

    pub fn get_pacing_factor(&self) -> Option<f64> {
        self.video_config.pacing_factor
    }
    pub fn use_alr_probing(&self) -> bool {
        self.video_config.alr_probing
    }

    pub fn libvpx_vp8_qp_max(&self) -> Option<i64> {
        if let Some(vp8_qp_max) = self.video_config.vp8_qp_max {
            if !(0..=63).contains(&vp8_qp_max) {
                tracing::warn!("Unsupported vp8_qp_max_ value, ignored.");
                return None;
            }
        }
        self.video_config.vp8_qp_max
    }
    pub fn libvpx_vp8_min_pixels(&self) -> Option<i64> {
        if let Some(vp8_min_pixels) = self.video_config.vp8_min_pixels {
            if vp8_min_pixels < 1 {
                tracing::warn!("Unsupported vp8_min_pixels_ value, ignored.");
                return None;
            }
        }
        self.video_config.vp8_min_pixels
    }
    pub fn libvpx_vp8_trusted_rate_controller(&self) -> bool {
        self.video_config.trust_vp8
    }
    pub fn vp8_boost_base_layer_quality(&self) -> bool {
        self.video_config.vp8_s0_boost
    }
    pub fn vp8_dynamic_rate_settings(&self) -> bool {
        todo!();
    }
    pub fn libvpx_vp9_trusted_rate_controller(&self) -> bool {
        self.video_config.trust_vp9
    }
    pub fn vp9_dynamic_rate_settings(&self) -> bool {
        todo!();
    }

    pub fn vp8_base_heavy_tl3_rate_allocation(&self) -> bool {
        self.video_config.vp8_base_heavy_tl3_alloc
    }

    pub fn use_encoder_bitrate_adjuster(&self) -> bool {
        self.video_config.bitrate_adjuster
    }
    pub fn bitrate_adjuster_can_use_network_headroom(&self) -> bool {
        self.video_config.adjuster_use_headroom
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn settings_with_queue_size(queue_size_ms: Option<i64>) -> RateControlSettings {
        RateControlSettings::new(&FieldTrials {
            congestion_window: CongestionWindowConfig {
                queue_size_ms,
                ..Default::default()
            },
            ..Default::default()
        })
    }

    /// A queue size of zero still gives a congestion window, with pushback.
    #[test]
    fn zero_queue_size_gives_congestion_window() {
        let settings = settings_with_queue_size(Some(0));

        assert!(settings.use_congestion_window());
        assert_eq!(settings.get_congestion_window_additional_time_ms(), 0);
        assert!(settings.use_congestion_window_pushback());
    }

    /// Without a queue size, there is no congestion window.
    #[test]
    fn absent_queue_size_gives_no_congestion_window() {
        let settings = settings_with_queue_size(None);

        assert!(!settings.use_congestion_window());
        assert!(!settings.use_congestion_window_pushback());
    }

    fn settings_with_min_bitrate(min_bitrate_bps: Option<i64>) -> RateControlSettings {
        RateControlSettings::new(&FieldTrials {
            congestion_window: CongestionWindowConfig {
                min_bitrate_bps,
                ..Default::default()
            },
            ..Default::default()
        })
    }

    /// A minimum pushback bitrate of zero still gives pushback, without a floor.
    #[test]
    fn zero_min_bitrate_gives_pushback() {
        let settings = settings_with_min_bitrate(Some(0));

        assert!(settings.use_congestion_window_pushback());
        assert_eq!(
            settings.congestion_window_min_pushback_target_bitrate_bps(),
            0
        );
    }

    /// Without a minimum pushback bitrate, the congestion window stays, but
    /// there is no pushback.
    #[test]
    fn absent_min_bitrate_gives_no_pushback() {
        let settings = settings_with_min_bitrate(None);

        assert!(settings.use_congestion_window());
        assert!(!settings.use_congestion_window_pushback());
        assert_eq!(
            settings.congestion_window_min_pushback_target_bitrate_bps(),
            30000
        );
    }
}
