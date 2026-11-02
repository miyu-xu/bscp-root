// Copyright 2026 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Cargo entry for the official build.rs-generated protobuf modules.
include!(concat!(env!("OUT_DIR"), "/generated.rs"));
