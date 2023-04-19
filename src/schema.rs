use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct FilterOptions {
  pub page: Option<usize>,
  pub limit: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct ParamOptions {
  pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateAnimeSchema {
  pub title: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateAnimeSchema {
  pub title: Option<String>,
  pub description: Option<String>,
}
