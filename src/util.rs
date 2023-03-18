use anyhow::Result;
use once_cell::sync::OnceCell;
use snowflake::SnowflakeIdBucket;
use std::sync::Mutex;

/// Holds the Argon2 configuration used for password checks.
pub static ARGON2_CONFIG: OnceCell<argon2::Config> = OnceCell::new();

/// Hashes user input to the Argon2 hash. Slow!
pub fn argon2_hash(password: &str) -> Result<String> {
    Ok(argon2::hash_encoded(
        password.as_bytes(),
        std::env::var("VF_SALT")
            .expect("VF_SALT is unset")
            .as_bytes(),
        &ARGON2_CONFIG.get().expect("ARGON2_CONFIG is unset"),
    )?)
}

/// Verifies an Argon2 hash against a password.
pub fn argon2_verify(hash: &str, password: &str) -> Result<bool> {
    Ok(argon2::verify_encoded(hash, password.as_bytes())?)
}

/// Snowflake ID Bucket
/// Wrapped in mutex because the bucket serializes new IDs.
pub static SNOWFLAKE_BUCKET: OnceCell<Mutex<SnowflakeIdBucket>> = OnceCell::new();

pub fn snowflake_id() -> i64 {
    SNOWFLAKE_BUCKET
        .get()
        .expect("SNOWFLAKE_BUCKET is unset")
        .lock()
        .expect("SNOWFLAKE_BUCKET is unlockable")
        .get_id()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password() {
        std::env::set_var("VG_SALT", "Yya6#MEU6a7S3ZCPy@8yXq@h");
        ARGON2_CONFIG
            .set(argon2::Config::default())
            .expect("failed ARGON2_CONFIG");

        let password = "qRMFtvQ&_2Wi8bWu66aybpU!R✨";
        let hash = argon2_hash(password).expect("failed to hash");

        assert_eq!(hash, "$argon2i$v=19$m=4096,t=3,p=1$WXlhNiNNRVU2YTdTM1pDUHlAOHlYcUBo$BE3zzlJr3LdhNx3xbdxOsJEaW8bgcWuFRnI029BUTZw");
        assert!(argon2_verify(&hash, password).expect("failed to verify"));

        let password2 = "qRMFtvQ&_2Wi8bWu66aybpU!R";
        assert!(!argon2_verify(&hash, password2).expect("failed to verify"));
    }
}
