use binrw::{BinRead, binread, BinWrite, binwrite};
use std::io::Cursor;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;

#[binwrite]
#[bw(little)]
struct Request {
    version: u8,
    command: u8,
    no_results: u16,
    offset: u16,

    #[bw(calc = query.len() as u16)]
    query_len: u16,
    #[bw(map = |s: &String| s.as_bytes())]
    query: String,
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct Result {
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

#[binread]
#[derive(Debug)]
#[br(little)]
struct Response {
    version: u8,
    command: u8,
    total_results: u16,
    no_results: u16,
    #[br(count = no_results)]
    results: Vec<Result>,
}

fn main() -> binrw::BinResult<()> {
    let mut stream = UnixStream::connect("/tmp/cocomel.sock")?;

    let req = Request {
        version: 0,
        command: 1, // search
        no_results: 10,
        offset: 0,
        query: "test".to_string(),
    };

    let mut send_buf = Cursor::new(Vec::new());
    req.write(&mut send_buf)?;

    let req_bytes = send_buf.into_inner();
    stream.write_all(&req_bytes)?;

    let mut recv_buf = Vec::new();
    stream.read_to_end(&mut recv_buf)?;

    let mut recv_cursor = Cursor::new(recv_buf);
    let res = Response::read(&mut recv_cursor)?;
    println!("{:?}", res);

    Ok(())
}
