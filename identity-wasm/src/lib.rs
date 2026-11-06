//! Narrow WebAssembly bridge for the shared Rust synthetic identity generator.
//!
//! The browser supplies a cryptographically random seed. All identifier
//! derivation and package/user scoping remain in identity-core.

use identity_core::{
    AppScope, decode_seed_hex, generate_android_id, generate_app_instance_id,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

const NOTICE: &str = "Synthetic Device Faker test data only. These values do not modify Android platform APIs, device hardware identifiers, telephony state, DRM provisioning, or hardware-backed attestation.";

#[derive(Debug, Serialize)]
struct DerivedIdentifiers {
    android_id: String,
    app_instance_id: String,
}

#[derive(Debug, Serialize)]
struct DerivedIdentityProfile {
    synthetic: bool,
    package: String,
    android_user_id: u32,
    seed_hex: String,
    identifiers: DerivedIdentifiers,
    notice: &'static str,
}

fn derive_profile(
    package: &str,
    android_user_id: u32,
    seed_hex: &str,
) -> Result<DerivedIdentityProfile, String> {
    let scope = AppScope::new(package, android_user_id)
        .map_err(|error| error.to_string())?;
    let seed = decode_seed_hex(seed_hex).map_err(|error| error.to_string())?;
    let android_id =
        generate_android_id(&seed, &scope).map_err(|error| error.to_string())?;
    let app_instance_id =
        generate_app_instance_id(&seed, &scope).map_err(|error| error.to_string())?;

    Ok(DerivedIdentityProfile {
        synthetic: true,
        package: package.to_owned(),
        android_user_id,
        seed_hex: seed_hex.to_ascii_lowercase(),
        identifiers: DerivedIdentifiers {
            android_id,
            app_instance_id,
        },
        notice: NOTICE,
    })
}

/// Derive a JSON preview using the shared Rust identity-core algorithm.
///
/// The seed must be 32 cryptographically random bytes represented as 64 hex
/// characters. The browser obtains those bytes from Web Crypto; no weak
/// randomness fallback exists in this crate.
#[wasm_bindgen]
pub fn derive_identity_json(
    package: &str,
    android_user_id: u32,
    seed_hex: &str,
) -> Result<String, JsValue> {
    let profile = derive_profile(package, android_user_id, seed_hex)
        .map_err(|error| JsValue::from_str(&error))?;

    serde_json::to_string(&profile)
        .map_err(|error| JsValue::from_str(&format!("JSON serialization failed: {error}")))
}

#[cfg(test)]
mod tests {
    use super::derive_profile;

    #[test]
    fn wasm_bridge_matches_the_shared_identity_core_fixed_vector() {
        let profile = derive_profile("com.example.game", 0, &"5a".repeat(32))
            .expect("valid fixed test profile");

        assert!(profile.synthetic);
        assert_eq!(profile.package, "com.example.game");
        assert_eq!(profile.android_user_id, 0);
        assert_eq!(profile.identifiers.android_id, "da970cdb547bb973");
        assert_eq!(
            profile.identifiers.app_instance_id,
            "6a8999ce-3a4e-8c23-aa6b-c80445b6bd7c"
        );
    }

    #[test]
    fn package_and_user_scope_are_validated() {
        assert!(derive_profile("not-a-package", 0, &"5a".repeat(32)).is_err());
        assert!(derive_profile("com.example.game", 0, "bad").is_err());
        assert!(derive_profile("com.example.game", 999, &"5a".repeat(32)).is_ok());
    }
}
