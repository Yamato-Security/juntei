// Copyright (C) 2026 Shirofune Security Co., Ltd.
//
// Unless explicitly acquired and licensed from Licensor under another license, the contents of this
// file are subject to the Reciprocal Public License ("RPL") Version 1.5, or subsequent versions as
// allowed by the RPL, and You may not copy or use this file in either source code or executable
// form, except in compliance with the terms and conditions of the RPL.
//
// All software distributed under the RPL is provided strictly on an "AS IS" basis, WITHOUT WARRANTY
// OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR HEREBY DISCLAIMS ALL SUCH WARRANTIES,
// INCLUDING WITHOUT LIMITATION, ANY WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE, QUIET ENJOYMENT, OR NON-INFRINGEMENT. See the RPL for specific language governing rights
// and limitations under the RPL.

//! Embeds the Windows file properties (Details tab) in juntei.exe: the version and the
//! descriptive values in `[package.metadata.winresource]` of Cargo.toml.
use std::{env, io};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // Build scripts run on the host, so check the target OS for cross-compilation.
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return Ok(());
    }

    // The version comes from Cargo.toml, the descriptive values from its
    // [package.metadata.winresource].
    let mut resource = winresource::WindowsResource::new();
    // English (US) version strings; this does not change the application language.
    resource.set_language(0x0409);

    if !env::var("CARGO_PKG_VERSION_PRE")
        .unwrap_or_default()
        .is_empty()
    {
        const VS_FF_PRERELEASE: u64 = 0x0000_0002;
        resource.set_version_info(winresource::VersionInfo::FILEFLAGS, VS_FF_PRERELEASE);
    }

    // Fail the build if the resource compiler cannot embed the metadata.
    resource.compile()
}
