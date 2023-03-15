use actix_web::web::Data;
use anyhow::Result;
use scylla::{FromRow, IntoTypedRows, Session};

#[derive(Debug, FromRow)]
pub struct Thread {
    pub id: i64,
    pub node_id: i64,
    pub bucket_id: i32,
    pub title: String,
    pub subtitle: Option<String>,
    pub first_post_id: i64,
    pub first_post_user_id: i64,
    pub last_post_id: i64,
    pub last_post_user_id: i64,
}

impl Thread {
    pub async fn fetch(scylla: Data<Session>, thread_id: i64) -> Result<Option<Self>> {
        if let Some(rows) = scylla
            .query(
                "SELECT
                    id,
                    node_id,
                    bucket_id,
                    title,
                    subtitle,
                    first_post_id,
                    first_post_user_id,
                    last_post_id,
                    last_post_user_id
                FROM volksforo.threads
                WHERE id = ?",
                (thread_id,),
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

    pub async fn fetch_node_page(
        scylla: Data<Session>,
        node_id: i64,
        bucket_id: i32,
    ) -> Result<Vec<Self>> {
        if let Some(rows) = scylla
            .query(
                "SELECT
                        id,
                        node_id,
                        bucket_id,
                        title,
                        subtitle,
                        first_post_id,
                        first_post_user_id,
                        last_post_id,
                        last_post_user_id
                    FROM volksforo.threads
                    WHERE node_id = ? AND bucket_id = ?",
                (node_id, bucket_id),
            )
            .await?
            .rows
        {
            let mut result = Vec::<Self>::with_capacity(rows.len());

            for row in rows.into_typed::<Self>() {
                let post = row?;
                result.insert(result.len(), post);
            }

            Ok(result)
        } else {
            Ok(Vec::default())
        }
    }
}

impl std::fmt::Display for Thread {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "<a href=\"/threads/{}/\">{}</a>", self.id, self.title)
    }
}
