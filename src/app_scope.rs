//! Helpers for resolving the app process scope during Zygisk specialization.

/// Android assigns each user a UID range of 100,000 IDs.
/// The user ID is the high-order portion of an application UID.
pub(crate) fn android_user_id(uid: i32) -> u32 {
    const AID_USER_OFFSET: u32 = 100_000;
    if uid <= 0 {
        return 0;
    }
    (uid as u32) / AID_USER_OFFSET
}

/// Resolve the base package name, preferring Android's app data directory.
///
/// nice_name may contain a process suffix (for example, :remote).
/// Returns None only when neither source supplies a non-empty name.
pub(crate) fn resolve_package_name(app_data_dir: &str, nice_name: &str) -> Option<String> {
    app_data_dir
        .rsplit('/')
        .next()
        .filter(|package| !package.is_empty())
        .or_else(|| nice_name.split(':').next().filter(|package| !package.is_empty()))
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::{android_user_id, resolve_package_name};

    #[test]
    fn android_user_id_uses_the_100000_uid_range() {
        assert_eq!(android_user_id(0), 0);
        assert_eq!(android_user_id(99_999), 0);
        assert_eq!(android_user_id(100_000), 1);
        assert_eq!(android_user_id(1_012_345), 10);
    }

    #[test]
    fn invalid_negative_uid_falls_back_to_primary_user() {
        assert_eq!(android_user_id(-1), 0);
    }

    #[test]
    fn package_name_prefers_app_data_directory() {
        assert_eq!(
            resolve_package_name("/data/user/10/com.example.game", "other.package"),
            Some("com.example.game".to_string())
        );
    }

    #[test]
    fn package_name_falls_back_to_nice_name_without_process_suffix() {
        assert_eq!(
            resolve_package_name("", "com.example.game:remote"),
            Some("com.example.game".to_string())
        );
    }

    #[test]
    fn package_name_returns_none_when_both_sources_are_empty() {
        assert_eq!(resolve_package_name("", ""), None);
        assert_eq!(resolve_package_name("/data/user/0/", ""), None);
    }
}
