/*
 *  Copyright (c) 2015 The WebRTC project authors. All Rights Reserved.
 *
 *  Use of this source code is governed by a BSD-style license
 *  that can be found in the LICENSE file in the root of the source
 *  tree. An additional intellectual property rights grant can be found
 *  in the file PATENTS.  All contributing project authors may
 *  be found in the AUTHORS file in the root of the source tree.
 */

//! The RTC event log and the events that goog_cc writes to it.
//!
//! The log mirrors libwebrtc's
//! [`api/rtc_event_log`](https://webrtc.googlesource.com/src/+/3075edeba1/api/rtc_event_log/).
//! The events mirror those of
//! [`logging/rtc_event_log/events`](https://webrtc.googlesource.com/src/+/3075edeba1/logging/rtc_event_log/events/)
//! that goog_cc logs, which the port keeps at that path too and this module
//! re-exports.

use std::fmt;

pub use crate::logging::rtc_event_log::events::{
    ProbeFailureReason, RtcEventAlrState, RtcEventBweUpdateDelayBased, RtcEventBweUpdateLossBased,
    RtcEventProbeClusterCreated, RtcEventProbeResultFailure, RtcEventProbeResultSuccess,
    RtcEventRemoteEstimate,
};

/// A log that the controller's components write [`RtcEvent`]s to.
///
/// It mirrors libwebrtc's `RtcEventLog`
/// ([`rtc_event_log.h#24`](https://webrtc.googlesource.com/src/+/3075edeba1/api/rtc_event_log/rtc_event_log.h#24)),
/// and [`NetworkControllerConfig::event_log`] gives it to the controller. The
/// port keeps only `Log`, because libwebrtc's `StartLogging` and `StopLogging`
/// manage the output of its log implementation.
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

/// An [`RtcEventLog`] that drops every event, libwebrtc's `RtcEventLogNull`
/// ([`rtc_event_log.h#58`](https://webrtc.googlesource.com/src/+/3075edeba1/api/rtc_event_log/rtc_event_log.h#58)).
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
