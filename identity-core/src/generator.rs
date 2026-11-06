use std::fmt;

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::AppScope;

type HmacSha256 = Hmac<Sha256>;

const ANDROID_ID_DOMAIN: &[u8] = b"device-faker/identity-core/android-id/v1\\0";
const APP_INSTANCE_ID_DOMAIN: &[u8] = b"device-faker/identity-core/app-instance-id/v1\\0";

/// Decode a 32-byte seed represented by exactly 64 hexadecimal characters.
pub fn decode_seed_hex(value: &str) -> Result<[u8; 32], DerivationError> {
    let input = value.as_bytes();
    if input.len() != 64 {
        return Err(DerivationError::InvalidSeedLength);
    }

    let mut seed = [0u8; 32];
    for (index, pair) in input.chunks_exact(2).enumerate() {
        let high = hex_nibble(pair[0]).ok_or(DerivationError::InvalidSeedCharacter)?;
        let low = hex_nibble(pair[1]).ok_or(DerivationError::InvalidSeedCharacter)?;
        seed[index] = (high << 4) | low;
    }

    Ok(seed)
}

fn hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn encode_hex(value: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(value.len() * 2);
    for byte in value {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn derive_digest(
    seed: &[u8; 32],
    scope: &AppScope,
    domain: &[u8],
) -> Result<[u8; 32], DerivationError> {
    let mut mac =
        HmacSha256::new_from_slice(seed).map_err(|_| DerivationError::KeyInitialization)?;

    mac.update(domain);

    let package = scope.package().as_bytes();
    let package_len =
        u32::try_from(package.len()).map_err(|_| DerivationError::ScopeEncoding)?;
    mac.update(&package_len.to_be_bytes());
    mac.update(package);
    mac.update(&scope.android_user_id().to_be_bytes());

    let result = mac.finalize().into_bytes();
    let mut digest = [0u8; 32];
    digest.copy_from_slice(&result);
    Ok(digest)
}

/// Derives a synthetic, app-scoped 64-bit hexadecimal value for test use.
///
/// This function does not change Settings.Secure.ANDROID_ID or guarantee that
/// an Android API returns this value.
pub fn generate_android_id(
    seed: &[u8; 32],
    scope: &AppScope,
) -> Result<String, DerivationError> {
    let digest = derive_digest(seed, scope, ANDROID_ID_DOMAIN)?;
    Ok(encode_hex(&digest[..8]))
}

/// Derives a deterministic app-instance identifier using UUID version 8.
///
/// This is a Device Faker synthetic identifier, not GAID, Firebase Installation ID,
/// Android ID, a carrier identifier, or a hardware-backed identifier.
pub fn generate_app_instance_id(
    seed: &[u8; 32],
    scope: &AppScope,
) -> Result<String, DerivationError> {
    let digest = derive_digest(seed, scope, APP_INSTANCE_ID_DOMAIN)?;
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);

    // RFC 9562 UUID version 8 custom-layout nibble.
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    // RFC variant: binary 10xx.
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    let hex = encode_hex(&bytes);
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32],
    ))
}

/// Errors returned while decoding a seed or deriving a scoped identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivationError {
    KeyInitialization,
    ScopeEncoding,
    InvalidSeedLength,
    InvalidSeedCharacter,
}

impl fmt::Display for DerivationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyInitialization => formatter.write_str("HMAC key initialization failed"),
            Self::ScopeEncoding => formatter.write_str("application scope is too large to encode"),
            Self::InvalidSeedLength => formatter.write_str("seed must contain exactly 64 hexadecimal characters"),
            Self::InvalidSeedCharacter => formatter.write_str("seed contains a non-hexadecimal character"),
        }
    }
}

impl std::error::Error for DerivationError {}
