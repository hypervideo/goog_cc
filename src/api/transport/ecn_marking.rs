/*
 *  Copyright 2024 The WebRTC Project Authors. All rights reserved.
 *
 *  Use of this source code is governed by a BSD-style license
 *  that can be found in the LICENSE file in the root of the source
 *  tree. An additional intellectual property rights grant can be found
 *  in the file PATENTS.  All contributing project authors may
 *  be found in the AUTHORS file in the root of the source tree.
 */

// TODO: bugs.webrtc.org/42225697 - L4S support is slowly being developed.
// Help is appreciated.

// L4S Explicit Congestion Notification (ECN) .
// https://www.rfc-editor.org/rfc/rfc9331.html ECT stands for ECN-Capable
// Transport and CE stands for Congestion Experienced.

// https://www.rfc-editor.org/rfc/rfc3168.html#section-5
// +-----+-----+
// | ECN FIELD |
// +-----+-----+
//   ECT   CE         [Obsolete] RFC 2481 names for the ECN bits.
//    0     0         Not-ECT
//    0     1         ECT(1)
//    1     0         ECT(0)
//    1     1         CE

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum EcnMarking {
    #[default]
    NotEct = 0b00, // Not ECN-Capable Transport
    Ect1 = 0b01, // ECN-Capable Transport
    Ect0 = 0b10, // Not used by L4S (or webrtc.)
    Ce = 0b11,   // Congestion experienced
}

impl EcnMarking {
    pub const fn as_str(self) -> &'static str {
        match self {
            EcnMarking::NotEct => "none",
            EcnMarking::Ect1 => "ect1",
            EcnMarking::Ect0 => "ect0",
            EcnMarking::Ce => "ce",
        }
    }
}

impl std::fmt::Display for EcnMarking {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
