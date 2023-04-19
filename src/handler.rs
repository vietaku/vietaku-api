use crate::{
  model::{AnimeModel, AnimeModelResponse},
  schema::{CreateAnimeSchema, FilterOptions, UpdateAnimeSchema},
  AppState,
};
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use serde_json::json;

#[get("/api/healthchecker")]
async fn health_checker_handler() -> impl Responder {
  const MESSAGE: &str = "Build Simple CRUD API with Rust, SQLX, MySQL, and Actix Web";

  HttpResponse::Ok().json(json!({"status": "success","message": MESSAGE}))
}

#[get("/animes")]
pub async fn anime_list_handler(
  opts: web::Query<FilterOptions>,
  data: web::Data<AppState>,
) -> impl Responder {
  let limit = opts.limit.unwrap_or(10);
  let offset = (opts.page.unwrap_or(1) - 1) * limit;

  let animes: Vec<AnimeModel> = sqlx::query_as!(
    AnimeModel,
    r#"SELECT * FROM animes ORDER by id LIMIT ? OFFSET ?"#,
    limit as i32,
    offset as i32
  )
  .fetch_all(&data.db)
  .await
  .unwrap();

  let anime_responses = animes
    .into_iter()
    .map(|anime| filter_db_record(&anime))
    .collect::<Vec<AnimeModelResponse>>();

  let json_response = serde_json::json!({
    "status": "success",
    "results": anime_responses.len(),
    "animes": anime_responses
  });
  HttpResponse::Ok().json(json_response)
}

#[post("/animes/")]
async fn create_anime_handler(
  body: web::Json<CreateAnimeSchema>,
  data: web::Data<AppState>,
) -> impl Responder {
  let user_id = uuid::Uuid::new_v4().to_string();
  let query_result =
    sqlx::query(r#"INSERT INTO animes (id, title, description) VALUES (?, ?, ?, ?)"#)
      .bind(user_id.clone())
      .bind(body.title.to_string())
      .bind(body.description.to_owned().unwrap_or_default())
      .execute(&data.db)
      .await
      .map_err(|err: sqlx::Error| err.to_string());

  if let Err(err) = query_result {
    if err.contains("Duplicate entry") {
      return HttpResponse::BadRequest().json(
        serde_json::json!({ "status": "fail", "message": "Anime with that title already exists" }),
      );
    }

    return HttpResponse::InternalServerError()
      .json(serde_json::json!({ "status": "error", "message": format!("{:?}", err) }));
  }

  let query_result = sqlx::query_as!(AnimeModel, r#"SELECT * FROM animes WHERE id = ?"#, user_id)
    .fetch_one(&data.db)
    .await;

  match query_result {
    Ok(anime) => {
      let anime_response = serde_json::json!({ "status": "success", "data": serde_json::json!({ "anime": filter_db_record(&anime) }) });

      return HttpResponse::Ok().json(anime_response);
    }
    Err(e) => {
      return HttpResponse::InternalServerError()
        .json(serde_json::json!({ "status": "error", "message": format!("{:?}", e) }));
    }
  }
}

#[get("/animes/{id}")]
async fn get_anime_handler(
  path: web::Path<uuid::Uuid>,
  data: web::Data<AppState>,
) -> impl Responder {
  let anime_id = path.into_inner().to_string();
  let query_result = sqlx::query_as!(AnimeModel, r#"SELECT * FROM animes WHERE id = ?"#, anime_id)
    .fetch_one(&data.db)
    .await;

  match query_result {
    Ok(anime) => {
      let anime_response = serde_json::json!({ "status": "success", "data": serde_json::json!({ "anime": filter_db_record(&anime) }) });

      return HttpResponse::Ok().json(anime_response);
    }
    Err(sqlx::Error::RowNotFound) => {
      return HttpResponse::NotFound().json(serde_json::json!({ "status": "fail", "message": format!("Anime with ID: {} not found", anime_id) }));
    }
    Err(e) => {
      return HttpResponse::InternalServerError()
        .json(serde_json::json!({ "status": "error", "message": format!("{:?}", e) }));
    }
  };
}

#[patch("/animes/{id}")]
async fn edit_anime_handler(
  path: web::Path<uuid::Uuid>,
  body: web::Json<UpdateAnimeSchema>,
  data: web::Data<AppState>,
) -> impl Responder {
  let anime_id = path.into_inner().to_string();
  let query_result = sqlx::query_as!(AnimeModel, r#"SELECT * FROM animes WHERE id = ?"#, anime_id)
    .fetch_one(&data.db)
    .await;

  let anime = match query_result {
    Ok(anime) => anime,
    Err(sqlx::Error::RowNotFound) => {
      return HttpResponse::NotFound().json(
                serde_json::json!({"status": "fail","message": format!("Anime with ID: {} not found", anime_id)}),
            );
    }
    Err(e) => {
      return HttpResponse::InternalServerError()
        .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
    }
  };

  let update_result = sqlx::query(r#"UPDATE animes SET title = ?, description = ? WHERE id = ?"#)
    .bind(body.title.to_owned().unwrap_or_else(|| anime.title.clone()))
    .bind(
      body
        .description
        .to_owned()
        .unwrap_or_else(|| anime.description.clone().unwrap()),
    )
    .bind(anime_id.to_owned())
    .execute(&data.db)
    .await;

  match update_result {
    Ok(result) => {
      if result.rows_affected() == 0 {
        let message = format!("Anime with ID: {} not found", anime_id);
        return HttpResponse::NotFound().json(json!({"status": "fail","message": message}));
      }
    }
    Err(e) => {
      let message = format!("Internal server error: {}", e);
      return HttpResponse::InternalServerError()
        .json(json!({"status": "error","message": message}));
    }
  }

  let updated_anime_result = sqlx::query_as!(
    AnimeModel,
    r#"SELECT * FROM animes WHERE id = ?"#,
    anime_id.to_owned()
  )
  .fetch_one(&data.db)
  .await;

  match updated_anime_result {
    Ok(anime) => {
      let anime_response = serde_json::json!({"status": "success","data": serde_json::json!({"anime": filter_db_record(&anime)})});

      HttpResponse::Ok().json(anime_response)
    }
    Err(e) => HttpResponse::InternalServerError()
      .json(serde_json::json!({"status": "error","message": format!("{:?}", e)})),
  }
}

#[delete("/animes/{id}")]
async fn delete_anime_handler(
  path: web::Path<uuid::Uuid>,
  data: web::Data<AppState>,
) -> impl Responder {
  let anime_id = path.into_inner().to_string();
  let query_result = sqlx::query!(r#"DELETE FROM animes WHERE id = ?"#, anime_id)
    .execute(&data.db)
    .await;

  match query_result {
    Ok(result) => {
      if result.rows_affected() == 0 {
        let message = format!("Anime with ID: {} not found", anime_id);
        HttpResponse::NotFound().json(json!({"status": "fail","message": message}))
      } else {
        HttpResponse::NoContent().finish()
      }
    }
    Err(e) => {
      let message = format!("Internal server error: {}", e);
      HttpResponse::InternalServerError().json(json!({"status": "error","message": message}))
    }
  }
}

fn filter_db_record(anime: &AnimeModel) -> AnimeModelResponse {
  AnimeModelResponse {
    id: anime.id.to_owned(),
    title: anime.title.to_owned(),
    description: anime.description.to_owned().unwrap(),
    createdAt: anime.created_at.unwrap(),
    updatedAt: anime.updated_at.unwrap(),
  }
}

pub fn config(conf: &mut web::ServiceConfig) {
  let scope = web::scope("/api")
    .service(health_checker_handler)
    .service(anime_list_handler)
    .service(create_anime_handler)
    .service(get_anime_handler)
    .service(edit_anime_handler)
    .service(delete_anime_handler);

  conf.service(scope);
}
