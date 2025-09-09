use password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use crate::random::random_bytes;

pub fn hash_password(password: &str) -> Result<String, String> {
    let ctx = Argon2::default();
    let salt: [u8; 16] = random_bytes(16).as_slice().try_into().unwrap();
    let salt_string = SaltString::encode_b64(&salt).unwrap();
    match ctx.hash_password(password.as_bytes(), &salt_string) {
        Ok(result) => Ok(result.to_string()),
        Err(_) => Err("Could not hash password".to_string())
    }
}


pub fn verify_password(password: &str, ciphertext: &str) -> Result<bool, String> {
    let ctx = Argon2::default();
    match PasswordHash::new(ciphertext) {
        Ok(hash) => {
            match ctx.verify_password(password.as_bytes(), &hash) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false)
            }
        },
        Err(_) => Err("Could Not Validate Password".to_string())
    }
}

#[cfg(test)]
pub mod password_hash_tests {
    use crate::passwords::{hash_password, verify_password};

    #[test]
    fn hash_a_password_correct_len() {
        let res = hash_password("superSecretPassword").unwrap();
        assert_ne!(res.len(), "superSecretPassword".len())
    }

    #[test]
    fn hash_a_password_ensure_unique() {
        let res = hash_password("wrong_password_for_result").unwrap();
        assert_ne!(res.as_str(), "$argon2id$v=19$m=19456,t=2,p=1$OGvJ/8bPLl5ReXN9h0uzZOESj6bGfv/K/6KIf9heg+s$3PDL8se7IfHxVNfIYikDxERwRtF9lNy5kxlSfwIWPJg")
    }

    #[test]
    fn verify_a_password() {
        match hash_password("superSecretPassword") {
            Ok(ph) => {
                match verify_password("superSecretPassword", ph.as_str()) {
                    Ok(verified) => { assert_eq!(verified, true) },
                    Err(_) => { assert_eq!(1, 2) }
                }
            },
            Err(_) => {
                assert_eq!(1, 2)
            }
        }
    }

    #[test]
    fn verify_a_password_invalid_hash_throws() {
        match hash_password("superSecretPassword") {
            Ok(ph) => {
                match verify_password("bogusPassword", ph.as_str()) {
                    Ok(verified) => { assert_eq!(verified, false) },
                    Err(_) => { assert_eq!(1, 2) }
                }
            },
            Err(_) => {
                assert_eq!(1, 2)
            }
        }
    }
}