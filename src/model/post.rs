use actix_web::web::Data;
use anyhow::Result;
use chrono::Duration;
use scylla::{FromRow, IntoTypedRows};
use std::collections::HashMap;
use tokio::task::JoinSet;
use uuid::Uuid;

// Define custom struct that matches User Defined Type created earlier
// wrapping field in Option will gracefully handle null field values
#[derive(Debug, FromRow)]
pub struct Post {
    pub id: i64,
    pub thread_id: i64,
    pub created_at: Duration,
    pub user_id: i64,
    pub ugc_id: Uuid,
}
#[derive(Debug, FromRow)]
pub struct PostPosition {
    pub thread_id: i64,
    pub position: i32,
    pub post_id: i64,
}

impl Post {
    pub async fn fetch(scylla: Data<scylla::Session>) -> Result<Vec<Self>> {
        if let Some(rows) = scylla
            .query(
                "SELECT id, thread_id, created_at, user_id, ugc_id FROM volksforo.posts",
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
    ) -> Result<(Vec<Self>, HashMap<i64, i32>)> {
        let start_pos = (page - 1) * 15;

        // Felix Mendes:
        // Scylla loves concurrency, the more you can keep all CPUs busy the better.
        // The problem with a single query to read from multiple partitions lies down on the fact that this single query will have to wait until
        // an entire page is filled to retrieve you back some results. As a result, you coordinator may become bottlenecked when you have other
        // spare CPUs/shards receiving no requests.
        // This is somewhat the same as explained under https://christopher-batey.blogspot.com/2015/02/cassandra-anti-pattern-misuse-of.html but for reads.
        let mut pos_queries = JoinSet::new();
        let mut post_queries = JoinSet::new();

        for n in 1..=15 {
            let nscylla = scylla.to_owned();
            pos_queries.spawn(async move {
                nscylla
                    .query(
                        r#"SELECT thread_id, position, post_id
                    FROM volksforo.post_positions
                    WHERE thread_id = ? AND position = ?
                    ;"#,
                        (thread_id, start_pos + n),
                    )
                    .await
            });
        }

        let mut positions = HashMap::with_capacity(15 * 2);
        while let Some(result) = pos_queries.join_next().await {
            if let Some(rows) = result??.rows {
                for row in rows.into_typed::<PostPosition>() {
                    let pos = row?;

                    // Queue up the post select while we're working on the other results. Fast!
                    let nscylla = scylla.to_owned();
                    post_queries.spawn(async move {
                        nscylla
                            .query(
                                r#"SELECT
                                    id,
                                    thread_id,
                                    created_at,
                                    user_id,
                                    ugc_id
                                FROM volksforo.posts
                                WHERE id = ?
                                ;"#,
                                (pos.post_id,),
                            )
                            .await
                    });

                    positions.insert(pos.post_id, pos.position);
                }
            }
        }

        let mut posts = Vec::<Post>::with_capacity(positions.len());
        while let Some(result) = post_queries.join_next().await {
            if let Some(rows) = result??.rows {
                for row in rows.into_typed::<Self>() {
                    posts.push(row?);
                }
            }
        }

        // results will be async so we gotta sort for integrity
        posts.sort_by(|a, b| a.id.cmp(&b.id));

        Ok((posts, positions))
    }
}
