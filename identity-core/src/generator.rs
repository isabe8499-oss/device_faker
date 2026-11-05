use std::fmt;

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::AppScope;

type HmacSha256 = Hmac<Sha256>;

const DOMAIN_SEPARATOR: &[u8] = b"device-faker/identity-core/app-instance-id/v1\0";

/// Derives a stable synthetic application-instance identifier.
///
/// The returned value is formatted as a UUID with the RFC 9562 version 8
/// and variant bits. It is deterministic for the same seed, package, and
/// Android user ID. It is **not** a random UUIDv4, GAID, Firebase ID, Android
/// ID, carrier identifier, or hardware-backed identifier.
pub fn generate_app_instance_id(
    seed: &[u8; 32],
    scope: &AppScope,
) -> Result<String, DerivationError> {
    let mut mac =
        HmacSha256::new_from_slice(seed).map_err(|_| DerivationError::KeyInitialization)?;

    mac.update(DOMAIN_SEPARATOR);

    let package = scope.package().as_bytes();
    let package_len = u32::try_from(package.len())
        .map_err(|_| DerivationError::ScopeEncoding)?;
    mac.update(&package_len.to_be_bytes());
    mac.update(package);
    mac.update(&scope.android_user_id().to_be_bytes());

    let digest = mac.finalize().into_bytes();
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&digest[..16]);

    // UUID version 8 is intended for custom UUID layouts.
    // The value remains deterministic; these bits do not make it UUIDv4.
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    // RFC variant: binary 10xx.
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    Ok(format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15],
    ))
}

/// Failures that can occur while deriving a deterministic identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DerivationError {
    /// HMAC rejected the fixed-size seed.
    KeyInitialization,
    /// The scope cannot be encoded using the generator's length prefix.
    ScopeEncoding,
}

impl fmt::Display for DerivationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KeyInitialization => formatter.write_str("HMAC key initialization failed"),
            Self::ScopeEncoding => formatter.write_str("application scope is too large to encode"),
        }
    }
}

impl std::error::Error for DerivationError {}
