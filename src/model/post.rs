use actix_web::web::Data;
use anyhow::Result;
use chrono::Duration;
use scylla::cql_to_rust::FromRowError;
use scylla::query::Query;
use scylla::statement::{Consistency, SerialConsistency};
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
    pub user_id: Option<i64>,
    pub ugc_id: Uuid,
}
#[derive(Debug, FromRow)]
pub struct PostPosition {
    pub thread_id: i64,
    pub position: i64,
    pub post_id: i64,
}

impl Post {
    pub async fn fetch(scylla: Data<scylla::Session>, post_id: i64) -> Result<Option<Self>> {
        Ok(scylla
            .query(
                r#"SELECT
                    id,
                    thread_id,
                    created_at,
                    user_id,
                    ugc_id
                FROM volksforo.posts
                WHERE
                    post_id ?
                ;"#,
                (post_id,),
            )
            .await?
            .rows
            .unwrap_or_default()
            .into_typed::<Self>()
            .collect::<Result<Vec<Self>, FromRowError>>()?
            .pop())
    }

    pub async fn fetch_many(
        scylla: Data<scylla::Session>,
        post_ids: Vec<i64>,
    ) -> Result<Vec<Self>> {
        // Felix Mendes:
        // Scylla loves concurrency, the more you can keep all CPUs busy the better.
        // The problem with a single query to read from multiple partitions lies down on the fact that this single query will have to wait until
        // an entire page is filled to retrieve you back some results. As a result, you coordinator may become bottlenecked when you have other
        // spare CPUs/shards receiving no requests.
        // This is somewhat the same as explained under https://christopher-batey.blogspot.com/2015/02/cassandra-anti-pattern-misuse-of.html but for reads.
        let mut queries = JoinSet::new();

        for post_id in post_ids {
            let nscylla = scylla.to_owned();
            queries.spawn(async move {
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
                        (post_id,),
                    )
                    .await
            });
        }

        let mut posts = Vec::with_capacity(queries.len() * 2);
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

    pub async fn fetch_thread(
        scylla: Data<scylla::Session>,
        thread_id: i64,
        page: i64,
    ) -> Result<(Vec<Self>, HashMap<i64, i64>)> {
        let start_pos = (page - 1) * 15;
        if let Some(rows) = scylla
            .query(
                r#"SELECT thread_id, position, post_id
                    FROM volksforo.post_positions
                    WHERE thread_id = ? AND position > ? AND position < ?
                ;"#,
                (thread_id, start_pos - 1, start_pos + 15 + 1),
            )
            .await?
            .rows
        {
            let mut positions = HashMap::with_capacity(rows.len());
            for row in rows.into_typed::<PostPosition>() {
                let pos = row?;
                positions.insert(pos.post_id, pos.position);
            }

            let posts = Self::fetch_many(
                scylla,
                positions
                    .iter()
                    .map(|(post_id, _)| post_id.to_owned())
                    .collect(),
            )
            .await?;

            Ok((posts, positions))
        } else {
            Ok(Default::default())
        }
    }

    pub async fn insert(&self, scylla: Data<scylla::Session>) -> Result<i64> {
        let max_position = scylla
            .query(
                r#"SELECT
                    MAX(position)
                    FROM volksforo.post_positions
                    WHERE thread_id = ?
                ;"#,
                (&self.thread_id,),
            )
            .await?
            .rows
            .unwrap_or_default()
            .into_typed::<(i64,)>()
            .collect::<Result<Vec<(i64,)>, FromRowError>>()?
            .first()
            .map_or(0, |r| r.0);

        let position = max_position + 1;
        let timestamp = chrono::Utc::now().timestamp_millis();

        let mut ins_pos = Query::new(
            r#"INSERT INTO volksforo.post_positions (
                thread_id,
                post_id,
                position
            )
            VALUES (?, ?, ?);"#,
        );
        ins_pos.set_consistency(Consistency::One);
        ins_pos.set_serial_consistency(Some(SerialConsistency::Serial));

        let ins_post = Query::new(
            r#"INSERT INTO volksforo.posts (
                id,
                thread_id,
                created_at,
                user_id,
                ugc_id
            )
            VALUES (?, ?, ?, ?, ?);"#,
        );

        match tokio::join!(
            scylla.query(ins_pos, (&self.thread_id, &self.id, &position)),
            scylla.query(
                ins_post,
                (
                    &self.id,
                    &self.thread_id,
                    &timestamp,
                    &self.user_id,
                    &self.ugc_id,
                )
            )
        ) {
            (Ok(_), Ok(_)) => Ok(position),
            (Ok(_), Err(err)) => Err(err.into()),
            (Err(err), Ok(_)) => Err(err.into()),
            (Err(err), Err(_)) => Err(err.into()),
        }
    }
}
