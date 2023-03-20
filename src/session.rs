use crate::model::User;
use actix_web::web::Data;
use anyhow::Result;
use scylla::Session as ScyllaSession;
use uuid::Uuid;

/// Representation of the current web user, which may or may not be signed in.
#[derive(Debug)]
pub struct Visitor {
    pub user: Option<User>,
}

impl Visitor {
    pub async fn new_from_uuid(scylla: Data<ScyllaSession>, uuid: &Uuid) -> Result<Self> {
        match User::fetch_session(scylla.clone(), uuid).await? {
            Some(session) => {
                log::debug!("found sesssion");
                Ok(Visitor {
                    user: User::fetch(scylla, session.user_id).await?,
                })
            }
            None => {
                log::debug!("sesssion NOT FOUND");
                Ok(Self::default())
            }
        }
    }
}

impl Default for Visitor {
    fn default() -> Self {
        Self { user: None }
    }
}
