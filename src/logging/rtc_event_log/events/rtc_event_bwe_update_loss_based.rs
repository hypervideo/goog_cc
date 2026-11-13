/*
 *  Copyright (c) 2017 The WebRTC project authors. All Rights Reserved.
 *
 *  Use of this source code is governed by a BSD-style license
 *  that can be found in the LICENSE file in the root of the source
 *  tree. An additional intellectual property rights grant can be found
 *  in the file PATENTS.  All contributing project authors may
 *  be found in the AUTHORS file in the root of the source tree.
 */

/// The send-side estimate updated its target
/// ([`rtc_event_bwe_update_loss_based.h`](https://webrtc.googlesource.com/src/+/3075edeba1/logging/rtc_event_log/events/rtc_event_bwe_update_loss_based.h)).
///
/// The target is the rate of the loss-based side, capped by the delay-based
/// estimate and the configured maximum bitrate, and raised to the configured
/// minimum bitrate
/// ([`send_side_bandwidth_estimation.cc#557`](https://webrtc.googlesource.com/src/+/3075edeba1/modules/congestion_controller/goog_cc/send_side_bandwidth_estimation.cc#557),
/// [`#587`](https://webrtc.googlesource.com/src/+/3075edeba1/modules/congestion_controller/goog_cc/send_side_bandwidth_estimation.cc#587)). With
/// [`FieldTrials::disable_receiver_limit_caps_only`], the receiver's limit caps
/// it too.
///
/// It is logged when the target or the fraction of lost packets changes, and on
/// any other update of the target more than 5s after the last event.
///
/// [`FieldTrials::disable_receiver_limit_caps_only`]: crate::experiments::FieldTrials::disable_receiver_limit_caps_only
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcEventBweUpdateLossBased {
    pub bitrate_bps: i32,
    /// The fraction of packets lost, in 1/256ths.
    pub fraction_loss: u8,
    /// The packets expected since the last update of the fraction of lost
    /// packets.
    ///
    /// libwebrtc logs the count after a loss update resets it
    /// ([`send_side_bandwidth_estimation.cc#346`](https://webrtc.googlesource.com/src/+/3075edeba1/modules/congestion_controller/goog_cc/send_side_bandwidth_estimation.cc#346)),
    /// so it is 0 on each event that a loss update logs. The count updates the
    /// fraction once it reaches 20, so it is below 20 on the other events.
    pub total_packets: i32,
}
