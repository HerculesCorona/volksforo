use actix_web::web::Data;
use anyhow::Result;
use chrono::Duration;
use scylla::{frame::value, FromRow, IntoTypedRows, Session};
use std::collections::HashMap;
use tokio::task::JoinSet;

#[derive(Debug, FromRow)]
pub struct Thread {
    pub id: i64,
    pub node_id: i64,
    pub bucket_id: i32,
    pub title: String,
    pub subtitle: Option<String>,
    pub created_at: Duration,
    pub first_post_id: i64,
    pub first_post_user_id: i64,
    pub last_post_id: i64,
    pub last_post_user_id: i64,
}

impl Thread {
    /// Adds to a thread's view count without blocking.
    pub fn bump_view_count(&self, scylla: Data<Session>) {
        let nscylla = scylla.to_owned();
        let thread_id = self.id.to_owned();

        tokio::spawn(async move {
            nscylla
                .query(
                    "UPDATE volksforo.thread_views SET view_count = view_count + 1 WHERE id = ?",
                    (thread_id,),
                )
                .await
        });
    }

    ///  Returns a single thread.
    pub async fn fetch(scylla: Data<Session>, thread_id: &i64) -> Result<Option<Self>> {
        if let Some(rows) = scylla
            .query(
                "SELECT
                    id,
                    node_id,
                    bucket_id,
                    title,
                    subtitle,
                    created_at,
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

    /// Returns all threads for a forum page.
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
                        created_at,
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
                result.push(row?);
            }

            return Ok(result);
        }

        Ok(Default::default())
    }

    /// Fetches the reply count of many threads.
    pub async fn fetch_many_reply_count(
        scylla: Data<Session>,
        thread_ids: Vec<i64>,
    ) -> Result<HashMap<i64, i64>> {
        let mut queries = JoinSet::new();
        let mut counters = HashMap::<i64, i64>::with_capacity(thread_ids.len());

        for thread_id in thread_ids {
            let nscylla = scylla.to_owned();
            queries.spawn(async move {
                nscylla
                    .query(
                        "SELECT id, reply_count FROM volksforo.thread_replies WHERE id = ?",
                        (thread_id,),
                    )
                    .await
            });
        }

        while let Some(result) = queries.join_next().await {
            if let Some(rows) = result??.rows {
                for row in rows.into_typed::<(i64, value::Counter)>() {
                    let counter = row?;
                    counters.insert(counter.0, counter.1 .0);
                }
            }
        }

        Ok(counters)
    }

    /// Fetches the reply count of a single thread.
    pub async fn fetch_reply_count(scylla: Data<Session>, thread_id: i64) -> Result<Option<i64>> {
        if let Some(rows) = scylla
            .query(
                "SELECT id, reply_count FROM volksforo.thread_replies WHERE id = ?",
                (thread_id,),
            )
            .await?
            .rows
        {
            for row in rows.into_typed::<(i64, value::Counter)>() {
                return Ok(Some(row?.1 .0));
            }
        }

        Ok(None)
    }

    /// Fetches the view count of one thread.
    pub async fn fetch_view_count(scylla: Data<Session>, thread_id: i64) -> Result<Option<i64>> {
        if let Some(rows) = scylla
            .query(
                "SELECT id, view_count FROM volksforo.thread_views WHERE id = ?",
                (thread_id,),
            )
            .await?
            .rows
        {
            for row in rows.into_typed::<(i64, value::Counter)>() {
                return Ok(Some(row?.1 .0));
            }
        }

        Ok(None)
    }

    /// Fetches the view count of many threads.
    pub async fn fetch_many_view_count(
        scylla: Data<Session>,
        thread_ids: Vec<i64>,
    ) -> Result<HashMap<i64, i64>> {
        let mut queries = JoinSet::new();
        let mut counters = HashMap::<i64, i64>::with_capacity(thread_ids.len());

        for thread_id in thread_ids {
            let nscylla = scylla.to_owned();
            queries.spawn(async move {
                nscylla
                    .query(
                        "SELECT id, view_count FROM volksforo.thread_views WHERE id = ?",
                        (thread_id,),
                    )
                    .await
            });
        }

        while let Some(result) = queries.join_next().await {
            if let Some(rows) = result??.rows {
                for row in rows.into_typed::<(i64, value::Counter)>() {
                    let counter = row?;
                    counters.insert(counter.0, counter.1 .0);
                }
            }
        }

        Ok(counters)
    }
}
