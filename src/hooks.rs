use anyhow::Context;
use jni::{
    Env, EnvUnowned, jni_sig, jni_str,
    objects::{JClass, JValue},
    strings::JNIStr,
};

use crate::config::{IdentityConfig, MergedAppConfig, NumberSetting};

/// Build 字段的有效伪装值：与 config::field_value 相同的归一化——
/// 未设置/空串/`__DELETE__` 返回 None（跳过 hook），`__EMPTY__` 归一化为空串。
fn field_str(field: &Option<String>) -> Option<&str> {
    match field.as_deref() {
        Some(value) if !value.is_empty() && value != "__DELETE__" => {
            Some(if value == "__EMPTY__" { "" } else { value })
        }
        _ => None,
    }
}

/// Resolve a validated app-local legacy Build.SERIAL value.
///
/// Whitespace-only strings are treated as unset, matching IdentityConfig's
/// configured-field semantics. The existing special markers are preserved:
/// __EMPTY__ requests an empty field and __DELETE__ leaves the platform value.
fn legacy_build_serial(identity: Option<&IdentityConfig>) -> Option<&str> {
    let identity = identity?;
    if identity.validate().is_err() {
        return None;
    }

    let serial = identity.serial.as_deref()?;
    if serial.trim().is_empty() {
        return None;
    }

    match serial {
        "__DELETE__" => None,
        "__EMPTY__" => Some(""),
        value => Some(value),
    }
}

/// 根据合并配置 Hook android.os.Build 的静态字段。
pub fn hook_build_fields(
    env: &mut EnvUnowned,
    merged_config: &MergedAppConfig,
) -> anyhow::Result<()> {
    env.with_env(|jenv| -> Result<(), jni::errors::Error> {
        let build_class = jenv.find_class(jni_str!("android/os/Build"))?;

        // Build.SERIAL is the deprecated app-process static field. It is distinct
        // from Build.getSerial(), which calls DeviceIdentifiersPolicyService over
        // Binder and is intentionally not modified by this process-local hook.
        if let Some(serial) = legacy_build_serial(merged_config.identity.as_ref()) {
            set_build_field(jenv, &build_class, jni_str!("SERIAL"), serial)
                .map_err(|_e| jni::errors::Error::JniCall(jni::errors::JniError::Unknown))?;
        }

        if let Some(manufacturer) = field_str(&merged_config.manufacturer) {
            set_build_field(jenv, &build_class, jni_str!("MANUFACTURER"), manufacturer)
                .map_err(|_e| jni::errors::Error::JniCall(jni::errors::JniError::Unknown))?;
        }

        if let Some(brand) = field_str(&merged_config.brand) {
            set_build_field(jenv, &build_class, jni_str!("BRAND"), brand)
                .map_err(|_e| jni::errors::Error::JniCall(jni::errors::JniError::Unknown))?;
        }

        if let Some(model) = field_str(&merged_config.model) {
            set_build_field(jenv, &build_class, jni_str!("MODEL"), model)
                .map_err(|_e| jni::errors::Error::JniCall(jni::errors::JniError::Unknown))?;
        }

        if let Some(device) = field_str(&merged_config.device) {
            set_build_field(jenv, &build_class, jni_str!("DEVICE"), device)
                .map_err(|_e| jni::errors::Error::JniCall(jni::errors::JniError::Unknown))?;
        }

        if let Some(product) = field_str(&merged_config.product) {
            set_build_field(jenv, &build_class, jni_str!("PRODUCT"), product)
                .map_err(|_e| jni::errors::Error::JniCall(jni::errors::JniError::Unknown))?;
        }

        // HARDWARE 字段
        if let Some(hardware) = field_str(&merged_config.hardware) {
            set_build_field(jenv, &build_class, jni_str!("HARDWARE"), hardware)
                .map_err(|_e| jni::errors::Error::JniCall(jni::errors::JniError::Unknown))?;
        }

        // BOARD 字段
        if let Some(board) = field_str(&merged_config.board) {
            set_build_field(jenv, &build_class, jni_str!("BOARD"), board)
                .map_err(|_e| jni::errors::Error::JniCall(jni::errors::JniError::Unknown))?;
        }

        // SOC_MODEL 字段（API 29+ 存在；Build.SOC_MODEL 的属性来源是 ro.soc.model）
        if let Some(soc_model) = field_str(&merged_config.soc_model) {
            set_build_field(jenv, &build_class, jni_str!("SOC_MODEL"), soc_model)
                .map_err(|_e| jni::errors::Error::JniCall(jni::errors::JniError::Unknown))?;
        }

        if let Some(fingerprint) = field_str(&merged_config.fingerprint) {
            set_build_field(jenv, &build_class, jni_str!("FINGERPRINT"), fingerprint)
                .map_err(|_e| jni::errors::Error::JniCall(jni::errors::JniError::Unknown))?;
        }

        if let Some(build_id) = field_str(&merged_config.build_id) {
            set_build_field(jenv, &build_class, jni_str!("ID"), build_id)
                .map_err(|_e| jni::errors::Error::JniCall(jni::errors::JniError::Unknown))?;
        }

        // Build.DISPLAY 的属性来源是 ro.build.display.id（单一属性，无分区副本）
        if let Some(display_id) = field_str(&merged_config.display_id) {
            set_build_field(jenv, &build_class, jni_str!("DISPLAY"), display_id)
                .map_err(|_e| jni::errors::Error::JniCall(jni::errors::JniError::Unknown))?;
        }

        hook_version_fields(jenv, &build_class, merged_config)
            .map_err(|_e| jni::errors::Error::JniCall(jni::errors::JniError::Unknown))?;

        Ok(())
    })
    .resolve::<jni::errors::ThrowRuntimeExAndDefault>();
    Ok(())
}

fn hook_version_fields(
    env: &mut Env,
    _build_class: &JClass,
    merged_config: &MergedAppConfig,
) -> anyhow::Result<()> {
    let version_class = env
        .find_class(jni_str!("android/os/Build$VERSION"))
        .context("Failed to find Build.VERSION class")?;

    if let Some(android_version) = field_str(&merged_config.android_version) {
        set_build_field(env, &version_class, jni_str!("RELEASE"), android_version)?;
    }

    // `__DELETE__`（NumberSetting::Delete）不 hook 该字段：与字符串字段的
    // `__DELETE__` 语义一致——只由 delete_props 列表在 companion 侧删除属性，
    // Java 层字段保持系统原值。
    if let Some(sdk_int) = merged_config.sdk_int.and_then(NumberSetting::value) {
        set_build_int_field(env, &version_class, jni_str!("SDK_INT"), sdk_int as i32)?;
    }

    // Build.VERSION.INCREMENTAL 的属性来源是 ro.build.version.incremental
    if let Some(incremental) = field_str(&merged_config.incremental) {
        set_build_field(env, &version_class, jni_str!("INCREMENTAL"), incremental)?;
    }

    // Build.VERSION.SECURITY_PATCH 的属性来源是 ro.build.version.security_patch
    if let Some(security_patch) = field_str(&merged_config.security_patch) {
        set_build_field(
            env,
            &version_class,
            jni_str!("SECURITY_PATCH"),
            security_patch,
        )?;
    }

    Ok(())
}

fn set_build_field(
    env: &mut Env,
    build_class: &JClass,
    field_name: &JNIStr,
    value: &str,
) -> anyhow::Result<()> {
    let _field_id = env
        .get_static_field_id(build_class, field_name, jni_sig!("Ljava/lang/String;"))
        .with_context(|| "Failed to get field ID".to_string())?;

    let new_value = env
        .new_string(value)
        .with_context(|| format!("Failed to create string for {value}"))?;

    env.set_static_field(
        build_class,
        field_name,
        jni_sig!("Ljava/lang/String;"),
        JValue::Object(&new_value),
    )
    .with_context(|| "Failed to set field".to_string())?;

    Ok(())
}

fn set_build_int_field(
    env: &mut Env,
    build_class: &JClass,
    field_name: &JNIStr,
    value: i32,
) -> anyhow::Result<()> {
    let _field_id = env
        .get_static_field_id(build_class, field_name, jni_sig!("I"))
        .with_context(|| "Failed to get field ID".to_string())?;

    env.set_static_field(build_class, field_name, jni_sig!("I"), JValue::Int(value))
        .with_context(|| "Failed to set field".to_string())?;

    Ok(())
}


#[cfg(test)]
mod identity_serial_tests {
    use super::legacy_build_serial;
    use crate::config::{IdentityConfig, IdentityMode};

    fn static_serial(value: &str) -> IdentityConfig {
        IdentityConfig {
            mode: IdentityMode::Static,
            serial: Some(value.to_owned()),
            ..IdentityConfig::default()
        }
    }

    #[test]
    fn valid_serial_is_selected_for_the_legacy_field() {
        let identity = static_serial("SERIAL-TEST-001");
        assert_eq!(legacy_build_serial(Some(&identity)), Some("SERIAL-TEST-001"));
    }

    #[test]
    fn whitespace_only_serial_is_treated_as_unset() {
        let identity = static_serial("   ");
        assert_eq!(legacy_build_serial(Some(&identity)), None);
    }

    #[test]
    fn invalid_identity_block_is_ignored_as_a_whole() {
        let identity = IdentityConfig {
            mode: IdentityMode::Static,
            serial: Some("bad\\nserial".to_owned()),
            android_id: Some("not-hex".to_owned()),
            ..IdentityConfig::default()
        };
        assert_eq!(legacy_build_serial(Some(&identity)), None);
    }

    #[test]
    fn empty_marker_is_supported_and_delete_marker_is_ignored() {
        assert_eq!(
            legacy_build_serial(Some(&static_serial("__EMPTY__"))),
            Some("")
        );
        assert_eq!(
            legacy_build_serial(Some(&static_serial("__DELETE__"))),
            None
        );
    }

    #[test]
    fn absent_identity_does_not_override_the_field() {
        assert_eq!(legacy_build_serial(None), None);
    }
}
