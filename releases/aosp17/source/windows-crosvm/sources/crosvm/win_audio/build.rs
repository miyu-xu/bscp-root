// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

fn main() {
    println!("cargo:rerun-if-changed=third_party/r8brain");
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        // Upstream documents an empty R8BSRC_DECL for a non-DLL object.
        // Keep the same C ABI, compiling the fixed MIT source with the target
        // compiler instead of downloading a DLL and Microsoft's debug CRT.
        cc::Build::new()
            .cpp(true)
            .std("c++11")
            .define("R8BSRC_DECL", "")
            .file("third_party/r8brain/DLL/r8bsrc.cpp")
            .compile("r8brain");
    }
}
