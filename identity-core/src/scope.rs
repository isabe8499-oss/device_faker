use std::fmt;

/// A validated Android application package and Android user ID.
///
/// Construct this from the package name and the actual Android user ID
/// extracted for the target process. Do not include the config-key suffix
/// (for example, `@999`) in `package`; pass the numeric user ID separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppScope {
    package: String,
    android_user_id: u32,
}

impl AppScope {
    /// Creates a scope after validating an ASCII, dotted package name.
    ///
    /// Package segments must start with an ASCII letter or underscore and may
    /// then contain ASCII letters, digits, or underscores.
    pub fn new(package: impl Into<String>, android_user_id: u32) -> Result<Self, ScopeError> {
        let package = package.into();

        if package.is_empty() {
            return Err(ScopeError::EmptyPackage);
        }

        if !is_valid_package_name(&package) {
            return Err(ScopeError::InvalidPackage);
        }

        Ok(Self {
            package,
            android_user_id,
        })
    }

    /// Returns the validated package name.
    pub fn package(&self) -> &str {
        &self.package
    }

    /// Returns the Android user ID associated with the target process.
    pub fn android_user_id(&self) -> u32 {
        self.android_user_id
    }
}

fn is_valid_package_name(package: &str) -> bool {
    let mut segments = package.split('.');
    let first = match segments.next() {
        Some(value) => value,
        None => return false,
    };

    if !is_valid_segment(first) {
        return false;
    }

    let mut count = 1usize;
    for segment in segments {
        if !is_valid_segment(segment) {
            return false;
        }
        count += 1;
    }

    count >= 2
}

fn is_valid_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    (first.is_ascii_alphabetic() || first == '_')
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

/// Errors returned while creating an application scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeError {
    /// The package name is empty.
    EmptyPackage,
    /// The package name is not a valid dotted ASCII identifier.
    InvalidPackage,
}

impl fmt::Display for ScopeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPackage => formatter.write_str("package name is empty"),
            Self::InvalidPackage => formatter.write_str("package name is invalid"),
        }
    }
}

impl std::error::Error for ScopeError {}
