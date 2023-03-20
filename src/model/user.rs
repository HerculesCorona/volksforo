use super::Post;
use actix_web::web::Data;
use anyhow::Result;
use blake3::{self, Hasher};
use chrono::{DateTime, Duration, Utc};
use scylla::{FromRow, IntoTypedRows, Session};
use std::collections::HashMap;
use tokio::task::JoinSet;
use uuid::{uuid, Uuid};

#[derive(Debug, FromRow, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub username_normal: String,
    pub email: Option<String>,
    pub password: String,
    pub password_cipher: String,
}

#[derive(Debug, FromRow, Clone)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: i64,
    pub created_at: Duration,
    pub last_seen_at: Duration,
}

impl User {
    pub async fn create(
        scylla: Data<Session>,
        username: String,
        email: Option<String>,
        password: String,
    ) -> Result<Self> {
        let id = crate::util::snowflake_id().await?;
        let user = Self {
            id: id.to_owned(),
            username: username.to_owned(),
            username_normal: username.to_lowercase(),
            email,
            password: crate::util::argon2_hash(&password)?,
            password_cipher: "argon2".to_owned(),
        };
        user.insert(scylla).await?;
        Ok(user)
    }

    pub async fn create_session(&self, scylla: Data<Session>) -> Result<Uuid> {
        let uuid = Uuid::new_v4();
        let timestamp = chrono::Utc::now().timestamp_millis();

        scylla
            .query(
                r#"INSERT INTO volksforo.user_sessions
                    (id, user_id, created_at, last_seen_at)
                    VALUES (?, ?, ?, ?)
                ;"#,
                (&uuid, &self.id, timestamp, timestamp),
            )
            .await?;

        Ok(uuid)
    }

    pub async fn fetch(scylla: Data<Session>, id: i64) -> Result<Option<Self>> {
        if let Some(rows) = scylla
            .query(
                "SELECT
                    id,
                    username,
                    username_normal,
                    email,
                    password,
                    password_cipher
                FROM volksforo.users
                WHERE id = ?",
                (id,),
            )
            .await?
            .rows
        {
            for row in rows.into_typed::<Self>() {
                return Ok(Some(row?));
            }
        }

        Ok(None)
    }

    pub async fn fetch_by_username(
        scylla: Data<Session>,
        username: String,
    ) -> Result<Option<Self>> {
        if let Some(rows) = scylla
            .query(
                "SELECT
                    id,
                    username,
                    username_normal,
                    email,
                    password,
                    password_cipher
                FROM volksforo.users
                WHERE username_normal = ?",
                (username,),
            )
            .await?
            .rows
        {
            for row in rows.into_typed::<Self>() {
                return Ok(Some(row?));
            }
        }

        Ok(None)
    }

    pub async fn fetch_many(scylla: Data<Session>, ids: Vec<i64>) -> Result<HashMap<i64, Self>> {
        let mut queries = JoinSet::new();
        let mut models = HashMap::with_capacity(ids.len());

        for id in ids {
            let nscylla = scylla.to_owned();
            queries.spawn(async move {
                nscylla
                    .query(
                        r#"SELECT
                            id,
                            username,
                            username_normal,
                            email,
                            password,
                            password_cipher
                        FROM volksforo.users
                        WHERE id = ?
                        LIMIT 1
                        ;"#,
                        (id,),
                    )
                    .await
            });
        }

        while let Some(result) = queries.join_next().await {
            if let Some(rows) = result??.rows {
                for row in rows.into_typed::<Self>() {
                    let model = row?;
                    models.insert(model.id, model);
                }
            }
        }

        Ok(models)
    }

    pub async fn fetch_many_post_authors(
        scylla: Data<Session>,
        posts: &Vec<Post>,
    ) -> Result<HashMap<i64, Self>> {
        Self::fetch_many(
            scylla.clone(),
            posts.iter().map(|x| x.id.to_owned()).collect::<Vec<i64>>(),
        )
        .await
    }

    pub async fn fetch_session(scylla: Data<Session>, uuid: &Uuid) -> Result<Option<UserSession>> {
        if let Some(rows) = scylla
            .query(
                "SELECT
                    id,
                    user_id,
                    created_at,
                    last_seen_at
                FROM volksforo.user_sessions
                WHERE id = ?",
                (uuid,),
            )
            .await?
            .rows
        {
            for row in rows.into_typed::<UserSession>() {
                return Ok(Some(row?));
            }
        }

        Ok(None)
    }

    pub async fn insert(&self, scylla: Data<Session>) -> Result<()> {
        scylla
            .query(
                r#"INSERT INTO volksforo.users
                    (id, username, username_normal, email, password, password_cipher)
                    VALUES (?, ?, ?, ?, ?, ?)
                ;"#,
                (
                    &self.id,
                    &self.username,
                    &self.username_normal,
                    &self.email,
                    &self.password,
                    &self.password_cipher,
                ),
            )
            .await?;
        Ok(())
    }
}
