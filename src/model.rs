use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
#[allow(non_snake_case)]
pub struct AnimeModel {
  pub id: String,
  pub title: String,
  pub description: Option<String>,
  pub created_at: Option<chrono::DateTime<chrono::Utc>>,
  pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct AnimeModelResponse {
  pub id: String,
  pub title: String,
  pub description: String,
  pub createdAt: chrono::DateTime<chrono::Utc>,
  pub updatedAt: chrono::DateTime<chrono::Utc>,
}
