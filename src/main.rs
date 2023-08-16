use std::{env, path::Path};

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use once_cell::sync::Lazy;
use tera::{Context, Tera};
use tower_http::services::ServeDir;

static TEMPLATES: Lazy<Tera> = Lazy::new(|| Tera::new("templates/**/*.html").unwrap());

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("no .env");

    let app = Router::new()
        .route("/", get(index))
        .nest_service("/public", ServeDir::new("public"));

    let config = RustlsConfig::from_pem_file(
        Path::new(&env::var("CERT_PATH").unwrap()),
        Path::new(&env::var("KEY_PATH").unwrap()),
    )
    .await
    .unwrap();

    axum_server::bind_rustls(env::var("ADDRESS").unwrap().parse().unwrap(), config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

struct ErrorResponse(anyhow::Error);

impl<E: Into<anyhow::Error>> From<E> for ErrorResponse {
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("A server error occured: {}", self.0),
        )
            .into_response()
    }
}

async fn index() -> Result<Html<String>, ErrorResponse> {
    Ok(Html(TEMPLATES.render("index.html", &Context::new())?))
}
