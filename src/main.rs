mod cocomel;

use askama::Template;
use axum::{
    Router,
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

const LOG_LEVEL: &'static str = if cfg!(debug_assertions) {
    "tower_http=trace,axum=debug"
} else {
    "info"
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(LOG_LEVEL)),
        )
        .init();

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/search", get(search_handler))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    total_docs: u32,
}

async fn index_handler() -> impl IntoResponse {
    let info_result = cocomel::info().unwrap();
    let template = IndexTemplate {
        total_docs: info_result.total_docs,
    };
    HtmlTemplate(template)
}

#[derive(Deserialize)]
struct Params {
    q: String,
    page: Option<usize>,
}

#[derive(Template)]
#[template(path = "results.html")]
struct ResultsTemplate {
    query: String,
    page: usize,
    total_results: usize,
    no_results: usize,
    results: Vec<cocomel::SearchResult>,
}

async fn search_handler(Query(params): Query<Params>) -> impl IntoResponse {
    let page = match params.page {
        Some(page) => {
            if page == 0 {
                1
            } else {
                page
            }
        }
        _ => 1,
    };
    let search_results = cocomel::search(&params.q, 10, page - 1).unwrap();
    let template = ResultsTemplate {
        query: params.q,
        page: page,
        total_results: search_results.total_results as usize,
        no_results: search_results.no_results as usize,
        results: search_results.results,
    };
    HtmlTemplate(template)
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}
