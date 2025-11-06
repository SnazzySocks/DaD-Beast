/// Unit tests for password module
///
/// Add to lib.rs:
/// #[cfg(test)]
/// mod password_tests;

#[cfg(test)]
mod tests {
    use super::super::password::*;

    #[test]
    fn test_hash_password() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).unwrap();

        // Hash should not be empty
        assert!(!hash.is_empty());

        // Hash should not equal password
        assert_ne!(hash, password);

        // Hash should start with $argon2 or $2b (depending on implementation)
        assert!(hash.starts_with('$'));
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "SecurePassword123!";
        let hash = hash_password(password).unwrap();

        assert!(verify_password(password, &hash).unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "SecurePassword123!";
        let wrong_password = "WrongPassword123!";
        let hash = hash_password(password).unwrap();

        assert!(!verify_password(wrong_password, &hash).unwrap());
    }

    #[test]
    fn test_hash_determinism() {
        let password = "TestPassword123!";

        // Same password should produce different hashes (due to salt)
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_validate_password_strength_weak() {
        let result = validate_password_strength("weak");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_no_uppercase() {
        let result = validate_password_strength("lowercase123!");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_no_lowercase() {
        let result = validate_password_strength("UPPERCASE123!");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_no_number() {
        let result = validate_password_strength("NoNumbers!");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_no_special() {
        let result = validate_password_strength("NoSpecial123");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength_strong() {
        let result = validate_password_strength("StrongPassword123!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_empty_password() {
        let result = hash_password("");
        assert!(result.is_err() || result.unwrap().is_empty() == false);
    }

    #[test]
    fn test_very_long_password() {
        let long_password = "A".repeat(1000) + "1!";
        let result = hash_password(&long_password);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unicode_password() {
        let unicode_password = "Пароль123!密码";
        let hash = hash_password(unicode_password).unwrap();
        assert!(verify_password(unicode_password, &hash).unwrap());
    }
}
