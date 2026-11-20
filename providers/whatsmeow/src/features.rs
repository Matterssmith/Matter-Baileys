/*
 * WhatsApp-Matter — Rust WhatsMeow parity ledger.
 * A feature is "verified" ONLY after a real isolated E2E test.
 *
 * Créditos:
 * Hecho por Lournyx Matterssmith [Lou-Nyx].
 */
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FeatureStatus {
    ImplementedOffline,
    UpstreamRustAvailableUnintegrated,
    RequiresImplementation,
    RequiresLiveTesting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Feature {
    pub name: &'static str,
    pub status: FeatureStatus,
    pub live_verified: bool,
}

pub const FEATURE_MATRIX: &[Feature] = &[
    Feature {
        name: "event_normalization",
        status: FeatureStatus::ImplementedOffline,
        live_verified: false,
    },
    Feature {
        name: "duplicate_event_filter",
        status: FeatureStatus::ImplementedOffline,
        live_verified: false,
    },
    Feature {
        name: "pairing_code",
        status: FeatureStatus::UpstreamRustAvailableUnintegrated,
        live_verified: false,
    },
    Feature {
        name: "session_persistence",
        status: FeatureStatus::UpstreamRustAvailableUnintegrated,
        live_verified: false,
    },
    Feature {
        name: "e2ee_signal",
        status: FeatureStatus::UpstreamRustAvailableUnintegrated,
        live_verified: false,
    },
    Feature {
        name: "dm_text_media",
        status: FeatureStatus::UpstreamRustAvailableUnintegrated,
        live_verified: false,
    },
    Feature {
        name: "groups_community",
        status: FeatureStatus::UpstreamRustAvailableUnintegrated,
        live_verified: false,
    },
    Feature {
        name: "presence_receipts",
        status: FeatureStatus::UpstreamRustAvailableUnintegrated,
        live_verified: false,
    },
    Feature {
        name: "contacts_privacy",
        status: FeatureStatus::UpstreamRustAvailableUnintegrated,
        live_verified: false,
    },
    Feature {
        name: "history_sync",
        status: FeatureStatus::UpstreamRustAvailableUnintegrated,
        live_verified: false,
    },
    Feature {
        name: "status_newsletters",
        status: FeatureStatus::UpstreamRustAvailableUnintegrated,
        live_verified: false,
    },
    Feature {
        name: "voice_video_calls",
        status: FeatureStatus::UpstreamRustAvailableUnintegrated,
        live_verified: false,
    },
    Feature {
        name: "message_edits_reactions",
        status: FeatureStatus::UpstreamRustAvailableUnintegrated,
        live_verified: false,
    },
    Feature {
        name: "polls_votes",
        status: FeatureStatus::RequiresLiveTesting,
        live_verified: false,
    },
    Feature {
        name: "nativeflow_buttons",
        status: FeatureStatus::RequiresLiveTesting,
        live_verified: false,
    },
    Feature {
        name: "webview_buttons",
        status: FeatureStatus::RequiresLiveTesting,
        live_verified: false,
    },
    Feature {
        name: "carousels_multicards",
        status: FeatureStatus::RequiresImplementation,
        live_verified: false,
    },
    Feature {
        name: "channels_newsletter_admin",
        status: FeatureStatus::RequiresImplementation,
        live_verified: false,
    },
    Feature {
        name: "anti_edit_evasion",
        status: FeatureStatus::RequiresImplementation,
        live_verified: false,
    },
    Feature {
        name: "socket_owner_authorization",
        status: FeatureStatus::RequiresImplementation,
        live_verified: false,
    },
    Feature {
        name: "vault_permissions",
        status: FeatureStatus::RequiresImplementation,
        live_verified: false,
    },
    Feature {
        name: "registration_https_card",
        status: FeatureStatus::RequiresImplementation,
        live_verified: false,
    },
    Feature {
        name: "sandbox_plugins",
        status: FeatureStatus::RequiresImplementation,
        live_verified: false,
    },
];

pub fn feature(name: &str) -> Option<&'static Feature> {
    FEATURE_MATRIX.iter().find(|feature| feature.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn no_unproven_live_feature_claims() {
        let mut names = HashSet::new();
        for feature in FEATURE_MATRIX {
            assert!(names.insert(feature.name), "duplicate capability");
            assert!(
                !feature.live_verified,
                "do not claim live capability without E2E"
            );
        }
        assert_eq!(
            feature("event_normalization").unwrap().status,
            FeatureStatus::ImplementedOffline
        );
        assert_eq!(feature("vault_permissions").unwrap().live_verified, false);
        assert!(feature("fictional").is_none());
    }
}
