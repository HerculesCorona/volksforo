use actix_web::web::Data;
use anyhow::Result;
use chrono::Duration;
use scylla::{FromRow, IntoTypedRows, Session};
use std::collections::HashMap;
use tokio::task::JoinSet;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct Ugc {
    pub id: Uuid,
    pub ip_id: Option<Uuid>,
    pub user_id: Option<i64>,
    pub created_at: Duration,
    pub content: String,
}

impl Ugc {
    pub async fn fetch_many(
        scylla: Data<Session>,
        uuids: Vec<Uuid>,
    ) -> Result<HashMap<Uuid, Self>> {
        let mut queries = JoinSet::new();
        let mut ugc = HashMap::with_capacity(uuids.len());

        for uuid in uuids {
            let nscylla = scylla.to_owned();
            queries.spawn(async move {
                nscylla
                    .query(
                        r#"SELECT
                            id,
                            ip_id,
                            user_id,
                            created_at,
                            content
                        FROM volksforo.ugc
                        WHERE id = ?
                        LIMIT 1
                        ;"#,
                        (uuid,),
                    )
                    .await
            });
        }

        while let Some(result) = queries.join_next().await {
            if let Some(rows) = result??.rows {
                for row in rows.into_typed::<Self>() {
                    let x = row?;
                    ugc.insert(x.id, x);
                }
            }
        }

        Ok(ugc)
    }

    pub async fn fetch_many_posts(
        scylla: Data<Session>,
        posts: &Vec<super::Post>,
    ) -> Result<HashMap<i64, Self>> {
        let uuids = posts
            .iter()
            .map(|x| x.ugc_id.to_owned())
            .collect::<Vec<Uuid>>();

        let mut ugc = Self::fetch_many(scylla, uuids).await?;
        let mut result = HashMap::with_capacity(ugc.len());

        for post in posts {
            if let Some(content) = ugc.remove(&post.ugc_id) {
                result.insert(post.id, content);
            }
        }

        Ok(result)
    }
}
