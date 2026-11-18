/*
 *  Copyright (c) 2019 The WebRTC project authors. All Rights Reserved.
 *
 *  Use of this source code is governed by a BSD-style license
 *  that can be found in the LICENSE file in the root of the source
 *  tree. An additional intellectual property rights grant can be found
 *  in the file PATENTS.  All contributing project authors may
 *  be found in the AUTHORS file in the root of the source tree.
 */

use crate::api::units::DataRate;

/// The controller got a new network state estimate
/// ([`rtc_event_remote_estimate.h`](https://webrtc.googlesource.com/src/+/3075edeba1/logging/rtc_event_log/events/rtc_event_remote_estimate.h)).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcEventRemoteEstimate {
    pub link_capacity_lower: DataRate,
    pub link_capacity_upper: DataRate,
}
