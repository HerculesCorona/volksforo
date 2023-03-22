use crate::session::Visitor;
use actix_web::web::Data;
use anyhow::Result;
use chrono::Duration;
use scylla::cql_to_rust::FromRowError;
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
    pub async fn create_for_visitor(
        scylla: Data<Session>,
        visitor: &Visitor,
        content: String,
    ) -> Result<Self> {
        let uuid = Uuid::new_v4();
        let user_id = visitor.user.as_ref().map(|u| Some(u.id));
        let timestamp = chrono::Utc::now().timestamp_millis();

        scylla
            .query(
                r#"INSERT INTO volksforo.ugc (
                    id,
                    ip_id,
                    user_id,
                    created_at,
                    content
                )
                VALUES (?, ?, ?, ?, ?);"#,
                (&uuid, None::<i64>, user_id, timestamp, content),
            )
            .await?;

        if let Some(ugc) = Self::fetch(scylla, &uuid).await? {
            Ok(ugc)
        } else {
            Err(anyhow::Error::new(crate::Error::Infallible(
                "Infallible select of UGC row we just inserted somehow returned no results.",
            )))
        }
    }

    pub async fn fetch(scylla: Data<scylla::Session>, uuid: &Uuid) -> Result<Option<Self>> {
        Ok(scylla
            .query(
                "SELECT id, ip_id, user_id, created_at, content FROM volksforo.ugc WHERE id = ?",
                (uuid,),
            )
            .await?
            .rows
            .unwrap_or_default()
            .into_typed::<Self>()
            .collect::<Result<Vec<Self>, FromRowError>>()?
            .pop())
    }

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
