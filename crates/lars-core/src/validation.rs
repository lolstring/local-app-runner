//! Input validation and sanitization for LARS
//!
//! This module provides security-critical validation functions to prevent
//! command injection and other security issues.

use crate::error::ValidationError;

/// Maximum length for service names
pub const MAX_NAME_LENGTH: usize = 64;

/// Validate a service name.
///
/// Service names must:
/// - Be 1-64 characters long
/// - Contain only alphanumeric characters, underscores, and hyphens
/// - Not be empty
///
/// # Examples
///
/// ```
/// use lars_core::validation::validate_service_name;
///
/// assert!(validate_service_name("my-service").is_ok());
/// assert!(validate_service_name("my_service_123").is_ok());
/// assert!(validate_service_name("").is_err());
/// assert!(validate_service_name("name; rm -rf /").is_err());
/// ```
pub fn validate_service_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError::InvalidNameLength(0));
    }

    if name.len() > MAX_NAME_LENGTH {
        return Err(ValidationError::InvalidNameLength(name.len()));
    }

    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ValidationError::InvalidNameCharacters);
    }

    Ok(())
}

/// Sanitize a string for safe shell usage.
///
/// This function:
/// - Rejects strings containing null bytes
/// - Uses shell_escape for proper escaping
///
/// # Examples
///
/// ```
/// use lars_core::validation::sanitize_for_shell;
///
/// assert!(sanitize_for_shell("hello world").is_ok());
/// assert!(sanitize_for_shell("hello\0world").is_err());
/// ```
pub fn sanitize_for_shell(input: &str) -> Result<String, ValidationError> {
    if input.contains('\0') {
        return Err(ValidationError::NullByteInInput);
    }

    Ok(shell_escape::escape(input.into()).to_string())
}

/// Validate that a string is not empty.
pub fn validate_not_empty(input: &str) -> Result<(), ValidationError> {
    if input.trim().is_empty() {
        return Err(ValidationError::EmptyInput);
    }
    Ok(())
}

/// Generate a valid service name from a command.
///
/// Extracts a meaningful name from the command:
/// - Skips environment variable assignments (VAR=value patterns)
/// - For npx/bunx commands, uses the package name (e.g., "npx vibe-kanban@latest" -> "vibe-kanban")
/// - Strips version suffixes (@latest, @1.0.0, etc.)
/// - Extracts binary name from paths (/usr/bin/python -> python)
///
/// # Examples
///
/// ```
/// use lars_core::validation::generate_service_name;
///
/// assert_eq!(generate_service_name("npm start"), "npm");
/// assert_eq!(generate_service_name("npx vibe-kanban@latest"), "vibe-kanban");
/// assert_eq!(generate_service_name("PORT=3000 npx my-app"), "my-app");
/// assert_eq!(generate_service_name("FOO=bar BAZ=qux python app.py"), "python");
/// assert_eq!(generate_service_name("/usr/bin/python script.py"), "python");
/// ```
pub fn generate_service_name(command: &str) -> String {
    // Collect all words that are NOT environment variable assignments
    let words: Vec<&str> = command
        .split_whitespace()
        .filter(|word| {
            // Skip if it looks like VAR=value (has = and starts with letter/underscore)
            if let Some(eq_pos) = word.find('=') {
                let before_eq = &word[..eq_pos];
                // Valid env var names: start with letter or underscore, contain only alphanumeric/_
                let is_env_var = !before_eq.is_empty()
                    && before_eq
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_ascii_alphabetic() || c == '_')
                    && before_eq
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_');
                !is_env_var
            } else {
                true
            }
        })
        .collect();

    if words.is_empty() {
        return "service".to_string();
    }

    // Extract just the binary name from path
    let executable = words[0].split('/').last().unwrap_or("service");

    // For npx/bunx/pnpx commands, use the package name (second argument)
    let name_source = if (executable == "npx" || executable == "bunx" || executable == "pnpx")
        && words.len() > 1
    {
        // Skip flags (start with -)
        words
            .iter()
            .skip(1)
            .find(|w| !w.starts_with('-'))
            .copied()
            .unwrap_or(executable)
    } else {
        executable
    };

    // Strip version suffix (@latest, @1.0.0, etc.) and scope prefix (@org/)
    let without_version = name_source.split('@').next().unwrap_or(name_source);
    let package_name = if without_version.is_empty() {
        // Scoped package like @org/package - get the part after @
        name_source
            .split('/')
            .last()
            .and_then(|s| s.split('@').next())
            .unwrap_or(name_source)
    } else {
        without_version
    };

    // Sanitize: keep only valid characters
    let sanitized: String = package_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .take(MAX_NAME_LENGTH)
        .collect();

    if sanitized.is_empty() {
        "service".to_string()
    } else {
        sanitized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_service_name_valid() {
        assert!(validate_service_name("valid_name").is_ok());
        assert!(validate_service_name("valid-name-123").is_ok());
        assert!(validate_service_name("a").is_ok());
        assert!(validate_service_name("ABC123").is_ok());
        assert!(validate_service_name("my-service").is_ok());
        assert!(validate_service_name("my_service").is_ok());
    }

    #[test]
    fn test_validate_service_name_empty() {
        let err = validate_service_name("").unwrap_err();
        assert!(matches!(err, ValidationError::InvalidNameLength(0)));
    }

    #[test]
    fn test_validate_service_name_too_long() {
        let long_name = "a".repeat(65);
        let err = validate_service_name(&long_name).unwrap_err();
        assert!(matches!(err, ValidationError::InvalidNameLength(65)));
    }

    #[test]
    fn test_validate_service_name_invalid_characters() {
        // Shell metacharacters should be rejected
        assert!(validate_service_name("name; rm -rf /").is_err());
        assert!(validate_service_name("name$(whoami)").is_err());
        assert!(validate_service_name("name`id`").is_err());
        assert!(validate_service_name("name|cat").is_err());
        assert!(validate_service_name("name&bg").is_err());
        assert!(validate_service_name("name>file").is_err());
        assert!(validate_service_name("name<file").is_err());
        assert!(validate_service_name("name'quoted'").is_err());
        assert!(validate_service_name("name\"quoted\"").is_err());
        assert!(validate_service_name("name with spaces").is_err());
        assert!(validate_service_name("name\ttab").is_err());
        assert!(validate_service_name("name\nnewline").is_err());
    }

    #[test]
    fn test_sanitize_for_shell_valid() {
        assert!(sanitize_for_shell("hello world").is_ok());
        assert!(sanitize_for_shell("echo 'hello'").is_ok());
    }

    #[test]
    fn test_sanitize_for_shell_null_byte() {
        let err = sanitize_for_shell("hello\0world").unwrap_err();
        assert!(matches!(err, ValidationError::NullByteInInput));
    }

    #[test]
    fn test_validate_not_empty() {
        assert!(validate_not_empty("hello").is_ok());
        assert!(validate_not_empty("").is_err());
        assert!(validate_not_empty("   ").is_err());
        assert!(validate_not_empty("\t\n").is_err());
    }

    #[test]
    fn test_generate_service_name() {
        assert_eq!(generate_service_name("echo hello"), "echo");
        assert_eq!(generate_service_name("/usr/bin/python script.py"), "python");
        assert_eq!(generate_service_name("npm start"), "npm");
        assert_eq!(generate_service_name(""), "service");
        assert_eq!(generate_service_name("   "), "service");

        // Should strip invalid characters
        let name = generate_service_name("test;evil");
        assert!(validate_service_name(&name).is_ok());
    }

    #[test]
    fn test_generate_service_name_skips_env_vars() {
        // Single env var prefix
        assert_eq!(generate_service_name("PORT=3000 npm start"), "npm");
        assert_eq!(generate_service_name("NODE_ENV=production node app.js"), "node");

        // Multiple env var prefixes
        assert_eq!(
            generate_service_name("FOO=bar BAZ=qux python app.py"),
            "python"
        );

        // Env var with path
        assert_eq!(
            generate_service_name("PATH=/usr/bin /usr/local/bin/ruby script.rb"),
            "ruby"
        );

        // Only env var, no command - returns default
        assert_eq!(generate_service_name("KEY=value"), "service");

        // Equal sign in the command itself (not a prefix env var)
        assert_eq!(
            generate_service_name("./configure --prefix=/usr"),
            "configure"
        );
    }

    #[test]
    fn test_generate_service_name_npx_commands() {
        // npx extracts package name
        assert_eq!(
            generate_service_name("npx vibe-kanban@latest"),
            "vibe-kanban"
        );
        assert_eq!(generate_service_name("npx create-react-app my-app"), "create-react-app");
        assert_eq!(generate_service_name("npx -y cowsay hello"), "cowsay");

        // With env vars
        assert_eq!(
            generate_service_name("PORT=50505 npx vibe-kanban@latest"),
            "vibe-kanban"
        );

        // bunx and pnpx work the same
        assert_eq!(generate_service_name("bunx my-tool"), "my-tool");
        assert_eq!(generate_service_name("pnpx some-package@1.0.0"), "some-package");

        // npx without package falls back to npx
        assert_eq!(generate_service_name("npx"), "npx");
        assert_eq!(generate_service_name("npx -y"), "npx");
    }
}
