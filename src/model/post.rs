use actix_web::web::Data;
use anyhow::Result;
use scylla::{FromRow, IntoTypedRows};
use tokio::task::JoinSet;
use uuid::Uuid;

// Define custom struct that matches User Defined Type created earlier
// wrapping field in Option will gracefully handle null field values
#[derive(Debug, FromRow)]
pub struct Post {
    pub id: i64,
    pub thread_id: i64,
    pub position: i32,
    pub user_id: i64,
    pub ugc_id: Uuid,
}

impl Post {
    pub async fn fetch(scylla: Data<scylla::Session>) -> Result<Vec<Self>> {
        if let Some(rows) = scylla
            .query(
                "SELECT id, thread_id, position, user_id, ugc_id FROM volksforo.posts",
                &[],
            )
            .await?
            .rows
        {
            let mut result = Vec::with_capacity(rows.len());
            for row in rows.into_typed::<Self>() {
                let post = row?;
                result.insert(result.len(), post);
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn fetch_thread(
        scylla: Data<scylla::Session>,
        thread_id: i64,
        page: i32,
    ) -> Result<Vec<Post>> {
        let start_pos = (page - 1) * 15;
        let mut queries = JoinSet::new();

        // Felix Mendes:
        // Scylla loves concurrency, the more you can keep all CPUs busy the better.
        // The problem with a single query to read from multiple partitions lies down on the fact that this single query will have to wait until
        // an entire page is filled to retrieve you back some results. As a result, you coordinator may become bottlenecked when you have other
        // spare CPUs/shards receiving no requests.
        // This is somewhat the same as explained under https://christopher-batey.blogspot.com/2015/02/cassandra-anti-pattern-misuse-of.html but for reads.
        for n in 1..=15 {
            let nscylla = scylla.to_owned();
            queries.spawn(async move {
                nscylla
                    .query(
                        r#"SELECT
                        id,
                        thread_id,
                        position,
                        user_id,
                        ugc_id
                    FROM volksforo.posts
                    WHERE
                        thread_id = ?
                        AND position = ?
                    ;"#,
                        (thread_id, start_pos + n),
                    )
                    .await
            });
        }

        let mut posts = Vec::with_capacity(15);
        while let Some(result) = queries.join_next().await {
            if let Some(rows) = result??.rows {
                for row in rows.into_typed::<Self>() {
                    posts.push(row?);
                }
            }
        }

        // results will be async so we gotta sort for integrity
        posts.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(posts)
    }
}
