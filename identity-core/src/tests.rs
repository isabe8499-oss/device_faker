use crate::{
    AppScope, ScopeError, decode_seed_hex, generate_android_id, generate_app_instance_id,
};

const SEED: [u8; 32] = [0x5a; 32];

fn scope(package: &str, user_id: u32) -> AppScope {
    AppScope::new(package, user_id).expect("test scope must be valid")
}

#[test]
fn same_seed_and_scope_produce_same_identifier() {
    let app = scope("com.example.game", 0);
    assert_eq!(
        generate_app_instance_id(&SEED, &app).unwrap(),
        generate_app_instance_id(&SEED, &app).unwrap()
    );
}

#[test]
fn different_packages_and_android_users_are_domain_separated() {
    let baseline = generate_app_instance_id(&SEED, &scope("com.example.game", 0)).unwrap();
    let other_package = generate_app_instance_id(&SEED, &scope("com.example.other", 0)).unwrap();
    let other_user = generate_app_instance_id(&SEED, &scope("com.example.game", 999)).unwrap();
    assert_ne!(baseline, other_package);
    assert_ne!(baseline, other_user);
}

#[test]
fn output_has_uuid_v8_shape_and_variant() {
    let value = generate_app_instance_id(&SEED, &scope("com.example.game", 0)).unwrap();
    let parts: Vec<&str> = value.split('-').collect();
    assert_eq!(
        parts.iter().map(|part| part.len()).collect::<Vec<_>>(),
        vec![8, 4, 4, 4, 12]
    );
    assert_eq!(parts[2].as_bytes()[0], b'8');
    assert!(matches!(parts[3].as_bytes()[0], b'8' | b'9' | b'a' | b'b'));
    assert!(value.chars().all(|ch| ch.is_ascii_hexdigit() || ch == '-'));
}

#[test]
fn android_id_is_16_hex_characters_and_deterministic() {
    let app = scope("com.example.game", 0);
    let first = generate_android_id(&SEED, &app).unwrap();
    let second = generate_android_id(&SEED, &app).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.len(), 16);
    assert!(first.bytes().all(|byte| byte.is_ascii_hexdigit()));
}

#[test]
fn identifier_kinds_use_separate_domains() {
    let app = scope("com.example.game", 0);
    let android_id = generate_android_id(&SEED, &app).unwrap();
    let instance_id = generate_app_instance_id(&SEED, &app).unwrap();
    let instance_prefix: String = instance_id.chars().filter(|ch| *ch != '-').take(16).collect();
    assert_ne!(android_id, instance_prefix);
}

#[test]
fn seed_hex_decodes_32_bytes_and_rejects_invalid_input() {
    assert_eq!(decode_seed_hex(&"5a".repeat(32)).unwrap(), SEED);
    assert!(decode_seed_hex("abcd").is_err());
    assert!(decode_seed_hex(&format!("{}z", "0".repeat(63))).is_err());
    assert!(decode_seed_hex(&format!(" {}", "0".repeat(63))).is_err());
}

#[test]
fn package_validation_rejects_empty_and_malformed_names() {
    assert_eq!(AppScope::new("", 0), Err(ScopeError::EmptyPackage));
    assert_eq!(AppScope::new("com.example..game", 0), Err(ScopeError::InvalidPackage));
    assert_eq!(AppScope::new("com.2example.game", 0), Err(ScopeError::InvalidPackage));
    assert_eq!(AppScope::new("com.example.game@999", 999), Err(ScopeError::InvalidPackage));
}

#[test]
fn package_validation_accepts_normal_dotted_names() {
    let app = AppScope::new("com.example_game.client2", 10).unwrap();
    assert_eq!(app.package(), "com.example_game.client2");
    assert_eq!(app.android_user_id(), 10);
}
