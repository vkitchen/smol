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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/search", get(search_handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

async fn index_handler() -> impl IntoResponse {
    let template = IndexTemplate;
    HtmlTemplate(template)
}

#[derive(Deserialize)]
struct Params {
    q: String,
}

#[derive(Template)]
#[template(path = "results.html")]
struct ResultsTemplate {
    query: String,
    total_results: u16,
    no_results: u16,
    results: Vec<cocomel::SearchResult>,
}

async fn search_handler(Query(params): Query<Params>) -> impl IntoResponse {
    let search_results = cocomel::search(&params.q, 10, 0).unwrap();
    let template = ResultsTemplate {
        query: params.q,
        total_results: search_results.total_results,
        no_results: search_results.no_results,
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
