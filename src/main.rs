#![allow(unused)]
use anyhow::Result;
use askama::Template;
use axum::extract::{Json as JsonExtract, Path};
use axum::response::{Html, IntoResponse, Result as AxumResult};
use axum::{
    Json, Router,
    http::StatusCode,
    routing::{get, get_service, post},
};
use diesel::prelude::*;
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::services::{ServeDir, ServeFile};

pub mod models;
pub mod schema;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[derive(Template)]
#[template(path = "index.html")]
struct HtmlTemplate {
    link: String,
}

#[derive(Deserialize)]
struct CreateLink {
    link: Option<String>,
    redirect: String,
}

async fn redirect(Path(request): Path<String>) -> AxumResult<impl IntoResponse> {
    use self::schema::links::dsl::*;
    let connection = &mut establish_connection();

    let results = links
        .filter(link.eq(&request))
        .load::<models::Link>(connection)
        .unwrap_or(vec![]);
    if results.is_empty() {
        return Ok((StatusCode::NOT_FOUND, "Link not found").into_response());
    }
    let result = &results[0];

    let x = HtmlTemplate {
        link: result.redirect.clone(),
    };
    AxumResult::Ok(Html(x.render().unwrap_or("rendering error".to_string())).into_response())
}

async fn new_link(JsonExtract(payload): JsonExtract<CreateLink>) -> AxumResult<impl IntoResponse> {
    use self::schema::links;

    let connection = &mut establish_connection();

    if payload.redirect.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Redirect URL is required").into());
    }

    let mut rand_string: String = {
        use rand::{Rng, distr::Alphanumeric};

        let rand_string: String = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect();
        rand_string
    };

    let new_link = models::Link {
        link: payload.link.clone().unwrap_or(rand_string.clone()),
        created_at: chrono::Utc::now().naive_utc(),
        redirect: payload.redirect.clone(),
    };

    if let Err(e) = diesel::insert_into(links::table)
        .values(&new_link)
        .execute(connection)
    {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
            .into());
    }

    Ok((
        StatusCode::CREATED,
        Json(json!({"message": payload.link.unwrap_or(rand_string)})),
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    let app = axum::Router::new()
        .route("/{request}", get(redirect))
        .route("/new", post(new_link))
        .route(
            "/styles.css",
            get_service(ServeFile::new("static/styles.css")),
        )
        .route(
            "/script.js",
            get_service(ServeFile::new("static/script.js")),
        )
        .route(
            "/index.html",
            get_service(ServeFile::new("static/homepage.html")),
        )
        .route("/", get_service(ServeFile::new("static/homepage.html")))
        .route(
            "/favicon.ico",
            get_service(ServeFile::new("static/favicon.ico")),
        )
        .route("/f", get_service(ServeDir::new("static/")));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
