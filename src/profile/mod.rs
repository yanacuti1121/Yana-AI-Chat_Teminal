// SPDX-FileCopyrightText: 2026 Vũ Văn Tâm
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileKind {
    Default,
    LocalOnly,
    Offline,
    LowMemory,
    Research,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeProfile {
    pub kind: ProfileKind,
    pub allow_network: bool,
    pub prefer_local_models: bool,
    pub max_context_tokens: u32,
    pub telemetry_capacity: usize,
}

impl RuntimeProfile {
    pub fn for_kind(kind: ProfileKind) -> Self {
        match kind {
            ProfileKind::Default => Self {
                kind,
                allow_network: true,
                prefer_local_models: true,
                max_context_tokens: 32_768,
                telemetry_capacity: 256,
            },
            ProfileKind::LocalOnly => Self {
                kind,
                allow_network: false,
                prefer_local_models: true,
                max_context_tokens: 16_384,
                telemetry_capacity: 128,
            },
            ProfileKind::Offline => Self {
                kind,
                allow_network: false,
                prefer_local_models: true,
                max_context_tokens: 8_192,
                telemetry_capacity: 64,
            },
            ProfileKind::LowMemory => Self {
                kind,
                allow_network: true,
                prefer_local_models: true,
                max_context_tokens: 4_096,
                telemetry_capacity: 32,
            },
            ProfileKind::Research => Self {
                kind,
                allow_network: true,
                prefer_local_models: false,
                max_context_tokens: 65_536,
                telemetry_capacity: 512,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_profile_disables_network() {
        let profile = RuntimeProfile::for_kind(ProfileKind::Offline);
        assert!(!profile.allow_network);
        assert!(profile.prefer_local_models);
    }

    #[test]
    fn low_memory_profile_bounds_context() {
        let profile = RuntimeProfile::for_kind(ProfileKind::LowMemory);
        assert_eq!(profile.max_context_tokens, 4_096);
    }
}
