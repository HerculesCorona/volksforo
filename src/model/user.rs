use super::Post;
use actix_web::web::Data;
use anyhow::Result;
use scylla::{FromRow, IntoTypedRows, Session};
use std::collections::HashMap;
use tokio::task::JoinSet;

#[derive(Debug, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: Option<String>,
    pub password: Option<String>,
}

impl User {
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
                            email,
                            password
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
}
