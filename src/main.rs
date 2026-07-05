use askama::Template;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};
use binrw::{BinRead, binread, BinWrite, binwrite};
use serde::Deserialize;
use std::io::Cursor;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;

#[binwrite]
#[bw(little)]
struct SearchRequest {
    version: u8,
    command: u8,
    no_results: u16,
    offset: u16,

    #[bw(calc = query.len() as u16)]
    query_len: u16,
    #[bw(map = |s: &String| s.as_bytes())]
    query: String,
}

#[allow(unused)]
#[binread]
#[br(little)]
struct SearchResult {
    docid_len: u16,
    #[br(count = docid_len, try_map = |bytes: Vec<u8>| String::from_utf8(bytes))]
    docid: String,

    title_len: u16,
    #[br(count = title_len, try_map = |bytes: Vec<u8>| String::from_utf8(bytes))]
    title: String,

    snippet_len: u16,
    #[br(count = snippet_len, try_map = |bytes: Vec<u8>| String::from_utf8(bytes))]
    snippet: String,
}

#[allow(unused)]
#[binread]
#[br(little)]
struct SearchResponse {
    version: u8,
    command: u8,
    total_results: u16,
    no_results: u16,
    #[br(count = no_results)]
    results: Vec<SearchResult>,
}

fn search(query: String) -> Result<SearchResponse, binrw::Error> {
    let mut stream = UnixStream::connect("/tmp/cocomel.sock")?;

    let req = SearchRequest {
        version: 0,
        command: 1, // search
        no_results: 10,
        offset: 0,
        query: query,
    };

    let mut send_buf = Cursor::new(Vec::new());
    req.write(&mut send_buf)?;

    let req_bytes = send_buf.into_inner();
    stream.write_all(&req_bytes)?;

    let mut recv_buf = Vec::new();
    stream.read_to_end(&mut recv_buf)?;

    let mut recv_cursor = Cursor::new(recv_buf);
    Ok(SearchResponse::read(&mut recv_cursor)?)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/search", get(search_handler));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
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
    total_results: u16,
    no_results: u16,
    results: Vec<SearchResult>,
}

async fn search_handler(Query(params): Query<Params>) -> impl IntoResponse {
    let search_results = search(params.q).unwrap();
    let template = ResultsTemplate{
        total_results: search_results.total_results,
        no_results: search_results.no_results,
        results: search_results.results,
    };
    HtmlTemplate(template)
}

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T> where T: Template, {
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
