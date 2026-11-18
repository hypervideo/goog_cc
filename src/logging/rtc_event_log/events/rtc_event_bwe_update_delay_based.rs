/*
 *  Copyright (c) 2017 The WebRTC project authors. All Rights Reserved.
 *
 *  Use of this source code is governed by a BSD-style license
 *  that can be found in the LICENSE file in the root of the source
 *  tree. An additional intellectual property rights grant can be found
 *  in the file PATENTS.  All contributing project authors may
 *  be found in the AUTHORS file in the root of the source tree.
 */

use crate::api::transport::BandwidthUsage;

/// The delay-based estimate or the state of its overuse detector changed
/// ([`rtc_event_bwe_update_delay_based.h`](https://webrtc.googlesource.com/src/+/3075edeba1/logging/rtc_event_log/events/rtc_event_bwe_update_delay_based.h)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcEventBweUpdateDelayBased {
    pub bitrate_bps: i32,
    pub detector_state: BandwidthUsage,
}
