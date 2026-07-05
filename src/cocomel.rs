use binrw::{BinRead, binread, BinWrite, binwrite};
use std::io::Cursor;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;

#[binwrite]
#[bw(little)]
struct SearchRequest<'a> {
    version: u8,
    command: u8,
    no_results: u16,
    offset: u16,

    #[bw(calc = query.len() as u16)]
    query_len: u16,
    #[bw(map = |&s| s.as_bytes())]
    query: &'a str,
}

#[allow(unused)]
#[binread]
#[br(little)]
pub struct SearchResult {
    docid_len: u16,
    #[br(count = docid_len, try_map = |bytes: Vec<u8>| String::from_utf8(bytes))]
    pub docid: String,

    title_len: u16,
    #[br(count = title_len, try_map = |bytes: Vec<u8>| String::from_utf8(bytes))]
    pub title: String,

    snippet_len: u16,
    #[br(count = snippet_len, try_map = |bytes: Vec<u8>| String::from_utf8(bytes))]
    pub snippet: String,
}

#[allow(unused)]
#[binread]
#[br(little)]
pub struct SearchResponse {
    version: u8,
    command: u8,
    pub total_results: u16,
    pub no_results: u16,
    #[br(count = no_results)]
    pub results: Vec<SearchResult>,
}

pub fn search(query: &str, results: usize, page: usize) -> Result<SearchResponse, binrw::Error> {
    let mut stream = UnixStream::connect("/tmp/cocomel.sock")?;

    let req = SearchRequest {
        version: 0,
        command: 1, // search
        no_results: results as u16,
        offset: (page * results) as u16,
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
