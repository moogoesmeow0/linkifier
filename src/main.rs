#![allow(unused)]
use std::env;

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
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger};
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
        warn!("Link not found: {}", request);
        return Ok((StatusCode::NOT_FOUND, "Link not found").into_response());
    }
    let result = &results[0];

    let x = HtmlTemplate {
        link: result.redirect.clone(),
    };

    info!("Redirecting {} to {}", request, result.redirect);
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
        warn!("Database insertion error: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
            .into());
    }

    info!(
        "Created new link: {} -> {}",
        new_link.link, new_link.redirect
    );

    Ok((
        StatusCode::CREATED,
        Json(json!({"message": payload.link.unwrap_or(rand_string)})),
    ))
}

#[tokio::main]
async fn main() -> Result<()> {
    let static_dir = env::var("STATIC_DIR").unwrap_or_else(|_| "static".to_string());
    println!("static dir: {}", static_dir);

    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        std::fs::File::create("/opt/linkifier/logs/app.log")?,
    )?;

    let app = axum::Router::new()
        .route("/{request}", get(redirect))
        .route("/new", post(new_link))
        .route(
            "/styles.css",
            get_service(ServeFile::new(format!("{}/styles.css", static_dir))),
        )
        .route(
            "/script.js",
            get_service(ServeFile::new(format!("{}/script.js", static_dir))),
        )
        .route(
            "/index.html",
            get_service(ServeFile::new(format!("{}/homepage.html", static_dir))),
        )
        .route(
            "/",
            get_service(ServeFile::new(format!("{}/homepage.html", static_dir))),
        )
        .route(
            "/favicon.ico",
            get_service(ServeFile::new(format!("{}/favicon.ico", static_dir))),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
