/*
 *  Copyright (c) 2017 The WebRTC project authors. All Rights Reserved.
 *
 *  Use of this source code is governed by a BSD-style license
 *  that can be found in the LICENSE file in the root of the source
 *  tree. An additional intellectual property rights grant can be found
 *  in the file PATENTS.  All contributing project authors may
 *  be found in the AUTHORS file in the root of the source tree.
 */

/// The probe bitrate estimator derived an estimate from the feedback of a probe
/// cluster
/// ([`rtc_event_probe_result_success.h`](https://webrtc.googlesource.com/src/+/3075edeba1/logging/rtc_event_log/events/rtc_event_probe_result_success.h)).
///
/// Like [`RtcEventProbeResultFailure`](super::RtcEventProbeResultFailure), it
/// can be logged several times for one cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtcEventProbeResultSuccess {
    pub id: i32,
    pub bitrate_bps: i32,
}
