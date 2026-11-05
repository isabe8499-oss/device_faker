use crate::{AppScope, ScopeError, generate_app_instance_id};

const SEED: [u8; 32] = [0x5a; 32];

fn scope(package: &str, user_id: u32) -> AppScope {
    AppScope::new(package, user_id).expect("test scope must be valid")
}

#[test]
fn same_seed_and_scope_produce_same_identifier() {
    let app = scope("com.example.game", 0);

    let first = generate_app_instance_id(&SEED, &app).expect("derivation should succeed");
    let second = generate_app_instance_id(&SEED, &app).expect("derivation should succeed");

    assert_eq!(first, second);
}

#[test]
fn different_packages_produce_different_identifiers() {
    let first = generate_app_instance_id(&SEED, &scope("com.example.game", 0))
        .expect("derivation should succeed");
    let second = generate_app_instance_id(&SEED, &scope("com.example.other", 0))
        .expect("derivation should succeed");

    assert_ne!(first, second);
}

#[test]
fn different_android_users_produce_different_identifiers() {
    let primary = generate_app_instance_id(&SEED, &scope("com.example.game", 0))
        .expect("derivation should succeed");
    let secondary = generate_app_instance_id(&SEED, &scope("com.example.game", 999))
        .expect("derivation should succeed");

    assert_ne!(primary, secondary);
}

#[test]
fn output_has_uuid_v8_shape_and_variant() {
    let value = generate_app_instance_id(&SEED, &scope("com.example.game", 0))
        .expect("derivation should succeed");

    let parts: Vec<&str> = value.split('-').collect();
    assert_eq!(parts.iter().map(|part| part.len()).collect::<Vec<_>>(), vec![8, 4, 4, 4, 12]);
    assert_eq!(parts[2].as_bytes()[0], b'8');
    assert!(matches!(parts[3].as_bytes()[0], b'8' | b'9' | b'a' | b'b'));
    assert!(value.chars().all(|ch| ch.is_ascii_hexdigit() || ch == '-'));
}

#[test]
fn package_validation_rejects_empty_and_malformed_names() {
    assert_eq!(AppScope::new("", 0), Err(ScopeError::EmptyPackage));
    assert_eq!(
        AppScope::new("com.example..game", 0),
        Err(ScopeError::InvalidPackage)
    );
    assert_eq!(
        AppScope::new("com.2example.game", 0),
        Err(ScopeError::InvalidPackage)
    );
    assert_eq!(
        AppScope::new("com.example.game@999", 999),
        Err(ScopeError::InvalidPackage)
    );
}

#[test]
fn package_validation_accepts_normal_dotted_names() {
    let app = AppScope::new("com.example_game.client2", 10)
        .expect("normal dotted package should be accepted");

    assert_eq!(app.package(), "com.example_game.client2");
    assert_eq!(app.android_user_id(), 10);
}
