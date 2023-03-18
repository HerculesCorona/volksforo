use super::Post;
use actix_web::web::Data;
use anyhow::Result;
use scylla::{FromRow, IntoTypedRows, Session};
use std::collections::HashMap;
use tokio::task::JoinSet;

#[derive(Debug, FromRow, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub username_normal: String,
    pub email: Option<String>,
    pub password: String,
    pub password_cipher: String,
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
            email: email,
            password: crate::util::argon2_hash(&password)?,
            password_cipher: "argon2".to_owned(),
        };
        user.insert(scylla).await?;
        Ok(user)
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
