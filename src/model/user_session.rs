use super::Post;
use actix_web::web::Data;
use anyhow::Result;
use chrono::Duration;
use scylla::{FromRow, IntoTypedRows, Session};
use std::collections::HashMap;
use tokio::task::JoinSet;
use uuid::Uuid;

#[derive(Debug, FromRow, Clone)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: i64,
    pub created_at: Duration,
    pub last_seen_at: Duration,
}

impl UserSession {
    // Increments the last_seen_at timestamp of a user session.
    pub async fn bump_last_seen_at(scylla: Data<Session>, uuid: &Uuid) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp_millis();

        scylla
            .query(
                "UPDATE volksforo.user_sessions SET last_seen_at = ? WHERE id = ?;",
                (&timestamp, uuid),
            )
            .await?;

        Ok(())
    }

    pub async fn fetch(scylla: Data<Session>, uuid: &Uuid) -> Result<Option<Self>> {
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
            for row in rows.into_typed::<Self>() {
                return Ok(Some(row?));
            }
        }

        Ok(None)
    }
}
