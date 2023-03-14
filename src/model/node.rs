use actix_web::web::Data;
use anyhow::Result;
use scylla::{FromRow, IntoTypedRows, Session};

#[derive(Debug, FromRow)]
pub struct Node {
    pub id: i64,
    pub display_order: i32,
    pub title: String,
    pub description: Option<String>,
}

impl Node {
    pub async fn fetch_all(scylla: Data<Session>) -> Result<Vec<Self>> {
        if let Some(rows) = scylla
            .query(
                "SELECT id, display_order, title, description FROM volksforo.nodes",
                &[],
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

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "<a href=\"{}\">{}</a>",
            format!("/forums/{}", self.id),
            self.title
        )
    }
}
