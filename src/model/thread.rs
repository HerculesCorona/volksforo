use actix_web::web::Data;
use anyhow::Result;
use scylla::{FromRow, IntoTypedRows, Session};

#[derive(Debug, FromRow)]
pub struct Thread {
    pub id: i64,
    pub node_id: i64,
    pub title: String,
    pub subtitle: Option<String>,
    pub first_post_id: i64,
    pub last_post_id: i64,
}

impl Thread {
    pub async fn fetch_node(scylla: Data<Session>, node_id: i64) -> Result<Vec<Self>> {
        if let Some(rows) = scylla
            .query(
                "SELECT id, node_id, title, subtitle, first_post_id, last_post_id
                FROM volksforo.threads
                WHERE node_id = ?",
                (node_id,),
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
        write!(
            f,
            "<a href=\"{}\">{}</a>",
            format!("/threads/{}", self.id),
            self.title
        )
    }
}
