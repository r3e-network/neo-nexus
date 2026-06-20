use std::env;

use super::validation::safe_fragment;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleasePackagePlatform {
    pub os: String,
    pub arch: String,
}

impl ReleasePackagePlatform {
    pub fn current() -> Self {
        Self {
            os: env::consts::OS.to_string(),
            arch: env::consts::ARCH.to_string(),
        }
    }

    pub(in crate::release_pack) fn id(&self) -> String {
        format!("{}-{}", safe_fragment(&self.os), safe_fragment(&self.arch))
    }
}

pub(in crate::release_pack) fn release_binary_name(platform: &ReleasePackagePlatform) -> String {
    if platform.os == "windows" {
        "neo-nexus.exe".to_string()
    } else {
        "neo-nexus".to_string()
    }
}
