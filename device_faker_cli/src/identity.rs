//! Synthetic app-scoped identity profile generation for local tests.
//!
//! Generated values are Device Faker test data. This module does not write
//! config.toml, install Android hooks, or alter any Android platform identifier.

use anyhow::{Result, anyhow};
use identity_core::{AppScope, decode_seed_hex, generate_android_id, generate_app_instance_id};

#[derive(Debug, Clone, PartialEq, Eq)]
struct GeneratedProfile {
    package_key: String,
    seed_hex: String,
    android_id: String,
    app_instance_id: String,
}

fn derive_profile(
    package: &str,
    android_user_id: u32,
    seed_hex: &str,
) -> Result<GeneratedProfile> {
    let scope = AppScope::new(package, android_user_id)
        .map_err(|error| anyhow!(error.to_string()))?;
    let seed = decode_seed_hex(seed_hex)
        .map_err(|error| anyhow!(error.to_string()))?;

    let android_id = generate_android_id(&seed, &scope)
        .map_err(|error| anyhow!(error.to_string()))?;
    let app_instance_id = generate_app_instance_id(&seed, &scope)
        .map_err(|error| anyhow!(error.to_string()))?;

    Ok(GeneratedProfile {
        package_key: format!("{package}@{android_user_id}"),
        seed_hex: seed_hex.to_ascii_lowercase(),
        android_id,
        app_instance_id,
    })
}

fn render_toml(profile: &GeneratedProfile) -> String {
    format!(
        r#"# Synthetic local test profile. These values do not change Android APIs.

[[apps]]
package = "{}"

[apps.identity]
mode = "seeded"
seed = "{}"

[apps.identity.generate]
android_id = true
app_instance_id = true
"#,
        profile.package_key,
        profile.seed_hex,
    )
}

fn render_json(profile: &GeneratedProfile) -> Result<String> {
    let value = serde_json::json!({
        "synthetic": true,
        "package": profile.package_key,
        "seed_hex": profile.seed_hex,
        "identifiers": {
            "android_id": profile.android_id,
            "app_instance_id": profile.app_instance_id,
        },
        "notice": "Synthetic Device Faker test data; no Android platform API is modified."
    });
    serde_json::to_string_pretty(&value).map_err(|error| anyhow!(error.to_string()))
}

/// Generate a profile and write it to stdout.
///
/// If seed_hex is absent, a 32-byte seed is requested from the operating
/// system CSPRNG. The seed is included in the output so the generated profile
/// can be reproduced; treat it as private profile material.
pub fn generate(
    package: &str,
    android_user_id: u32,
    seed_hex: Option<&str>,
    json: bool,
) -> Result<()> {
    let seed_hex = match seed_hex {
        Some(value) => {
            decode_seed_hex(value).map_err(|error| anyhow!(error.to_string()))?;
            value.to_ascii_lowercase()
        }
        None => {
            let mut seed = [0u8; 32];
            getrandom::fill(&mut seed)
                .map_err(|error| anyhow!("operating-system CSPRNG failed: {error}"))?;
            encode_hex(&seed)
        }
    };

    let profile = derive_profile(package, android_user_id, &seed_hex)?;

    if json {
        println!("{}", render_json(&profile)?);
    } else {
        print!("{}", render_toml(&profile));
    }

    eprintln!(
        "Generated synthetic test data for {}. Keep the emitted seed private; no Android identifier has been changed.",
        profile.package_key
    );
    Ok(())
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 0x0f) as usize] as char);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{derive_profile, render_json, render_toml};

    #[test]
    fn generated_toml_uses_exact_user_scoped_package_key() {
        let profile = derive_profile(
            "com.example.game",
            999,
            &"5a".repeat(32),
        ).expect("profile derivation should succeed");

        let toml = render_toml(&profile);
        assert!(toml.contains("package = \"com.example.game@999\""));
        assert!(toml.contains("mode = \"seeded\""));
        assert!(toml.contains("android_id = true"));
        assert!(toml.contains("app_instance_id = true"));
    }

    #[test]
    fn generated_json_is_explicitly_synthetic_and_deterministic() {
        let profile = derive_profile(
            "com.example.game",
            0,
            &"5a".repeat(32),
        ).expect("profile derivation should succeed");

        let json = render_json(&profile).expect("JSON serialization should succeed");
        assert!(json.contains("\"synthetic\": true"));
        assert!(json.contains("da970cdb547bb973"));
        assert!(json.contains("6a8999ce-3a4e-8c23-aa6b-c80445b6bd7c"));
    }

    #[test]
    fn invalid_seed_and_package_are_rejected() {
        assert!(derive_profile("com.example.game", 0, "bad").is_err());
        assert!(derive_profile("not-a-package", 0, &"5a".repeat(32)).is_err());
    }
}
