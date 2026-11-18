/*
 *  Copyright (c) 2017 The WebRTC project authors. All Rights Reserved.
 *
 *  Use of this source code is governed by a BSD-style license
 *  that can be found in the LICENSE file in the root of the source
 *  tree. An additional intellectual property rights grant can be found
 *  in the file PATENTS.  All contributing project authors may
 *  be found in the AUTHORS file in the root of the source tree.
 */

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

/// Why the probe bitrate estimator rejected the feedback of a probe cluster,
/// libwebrtc's `ProbeFailureReason`
/// ([`rtc_event_probe_result_failure.h#28`](https://webrtc.googlesource.com/src/+/3075edeba1/logging/rtc_event_log/events/rtc_event_probe_result_failure.h#28)).
///
/// libwebrtc's enum also has `kTimeout`, which goog_cc does not log.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeFailureReason {
    /// The cluster's packets were sent or received over an interval of at most
    /// zero or of more than 1s.
    InvalidSendReceiveInterval,
    /// The cluster's packets were received at more than twice the rate they
    /// were sent at.
    InvalidSendReceiveRatio,
}
