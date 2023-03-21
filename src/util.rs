use anyhow::Result;
use askama_actix::Template;
use once_cell::sync::OnceCell;
use std::ops::Range;

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

/// Normalize a username from user input.
pub fn normalize_username(username: &str) -> String {
    username.trim().to_lowercase()
}

/// Snowflake ID Bucket
/// Wrapped in mutex because the bucket serializes new IDs.
pub static SNOWFLAKE_BUCKET: OnceCell<hexafreeze::Generator> = OnceCell::new();

pub async fn snowflake_id() -> Result<i64> {
    Ok(SNOWFLAKE_BUCKET
        .get()
        .expect("SNOWFLAKE_BUCKET is unset")
        .generate()
        .await?)
}

const PAGINATOR_LOOK_AHEAD: i64 = 2;

/// [1] 2 3 ... 13
/// 1 2 [3] 4 5 ... 13
/// 1 2 3 4 [5] 6 7 ... 13
/// 1 ... 4 5 [6] 7 8 ... 13
/// 1 ... 7 8 [9] 10 11 12 13
/// 1 ... 9 10 [11] 12 13
/// 1 ... 11 12 [13]
#[derive(Debug)]
pub struct Paginator {
    pub base_url: String,
    pub this_page: i64,
    pub page_count: i64,
}

#[derive(Template)]
#[template(path = "util/paginator.html")]
struct PaginatorTemplate<'a> {
    paginator: &'a Paginator,
}

pub trait PaginatorToHtml {
    fn as_html(&self) -> String;
    fn has_pages(&self) -> bool;
    fn get_first_pages(&self) -> Range<i64>;
    fn get_inner_pages(&self) -> Option<Range<i64>>;
    fn get_last_pages(&self) -> Option<Range<i64>>;
}

impl PaginatorToHtml for Paginator {
    fn has_pages(&self) -> bool {
        self.page_count > 1
    }

    fn get_first_pages(&self) -> Range<i64> {
        if 1 + PAGINATOR_LOOK_AHEAD < self.this_page - PAGINATOR_LOOK_AHEAD {
            // if 1+lookahead is less than page-lookahead, we only show page 1
            // i.e. any page starting with 6
            1..1
        } else if self.this_page + PAGINATOR_LOOK_AHEAD < self.page_count - PAGINATOR_LOOK_AHEAD {
            // if our lookahead is less than the lookbehind of the last page, show up to our lookahead.
            // i.e. on page 4 of 9, show 1-6 ... 9
            1..(self.this_page + PAGINATOR_LOOK_AHEAD)
        } else {
            // otherwise, just show all pages.
            // i.e. 5 of 9 is the greatest extent possible
            1..self.page_count
        }
    }

    fn get_inner_pages(&self) -> Option<Range<i64>> {
        // if our lookahead is gt/eq the lookbehind of the last page, we merge our cursor to the last pages
        if (1 + PAGINATOR_LOOK_AHEAD >= self.this_page - PAGINATOR_LOOK_AHEAD) ||
            // if 1+lookahead is less than page-lookahead, we only have first pages
            (self.this_page + PAGINATOR_LOOK_AHEAD >= self.page_count - PAGINATOR_LOOK_AHEAD)
        {
            None
        } else {
            // otherwise, show the lookahead and look behind
            // i.e. 1 .. 4 5 [6] 7 8 .. 11 (minimum number)
            Some((self.this_page - PAGINATOR_LOOK_AHEAD)..(self.this_page + PAGINATOR_LOOK_AHEAD))
        }
    }

    fn get_last_pages(&self) -> Option<Range<i64>> {
        if 1 + PAGINATOR_LOOK_AHEAD >= self.this_page - PAGINATOR_LOOK_AHEAD {
            // if 1+lookahead is less than page-lookahead, we only have first pages
            None
        } else if self.this_page + PAGINATOR_LOOK_AHEAD < self.page_count - PAGINATOR_LOOK_AHEAD {
            // if our lookahead is less than the lookbehind of the last page, show the last page
            Some(self.page_count..self.page_count)
        } else {
            // otherwise, show from the lookbehind of the cursor to the last page.
            Some((self.this_page - PAGINATOR_LOOK_AHEAD)..self.page_count)
        }
    }

    fn as_html(&self) -> String {
        if self.has_pages() {
            let mut buffer = String::new();
            let template = PaginatorTemplate { paginator: self };
            if template.render_into(&mut buffer).is_err() {
                "[Paginator Util Error]".to_owned()
            } else {
                buffer
            }
        } else {
            String::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password() {
        std::env::set_var("VF_SALT", "Yya6#MEU6a7S3ZCPy@8yXq@h");
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

    #[test]
    fn test_id() {
        std::env::set_var("VF_MACHINE_ID", "1");
    }
}
