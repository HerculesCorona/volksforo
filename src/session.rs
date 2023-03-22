use crate::model::{User, UserSession};
use actix_web::web::Data;
use anyhow::Result;
use scylla::Session as ScyllaSession;
use uuid::Uuid;

/// Representation of the current web user, which may or may not be signed in.
#[derive(Debug, Default)]
pub struct Visitor {
    pub session_id: Option<Uuid>,
    pub user: Option<User>,
}

impl Visitor {
    pub async fn new_from_uuid(scylla: Data<ScyllaSession>, uuid: Uuid) -> Result<Self> {
        match UserSession::fetch(scylla.clone(), &uuid).await? {
            Some(session) => Ok(Visitor {
                session_id: Some(uuid),
                user: User::fetch(scylla, session.user_id).await?,
            }),
            None => {
                log::debug!("Requested session not found: {}", uuid);
                Ok(Self::default())
            }
        }
    }
}
