use std::{
    fs,
    io::{BufReader, Error, ErrorKind, prelude::*},
    net::{TcpListener, TcpStream},
    str::FromStr,
    sync::Arc,
    thread,
};

fn main() {
    let listener = TcpListener::bind("[::]:31233").unwrap();

    let valid_pages = Arc::new(get_pages());

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let pages_ref = Arc::clone(&valid_pages);
        thread::spawn(move || {
            handle_connection(stream, pages_ref);
        });
    }
}

fn handle_connection(mut stream: TcpStream, pages: Arc<Vec<String>>) -> Result<String, Error> {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader
        .lines()
        .next()
        .ok_or(Error::new(ErrorKind::Other, "no lines"))??;

    let req = parse_http(&request_line[..]);

    println!("Requset received: {request_line}");

    let mut status_line = "HTTP/1.1 200 OK";
    let contents;

    if pages.contains(&req.path) {
        let mut pages_path = String::from_str("pages/").unwrap();
        pages_path.push_str(&req.path[..]);
        contents = fs::read_to_string(&pages_path).unwrap();
    } else if &req.path == "" {
        contents = fs::read_to_string("pages/home").unwrap();
    } else {
        status_line = "HTTP/1.1 404 NOT FOUND";
        contents = fs::read_to_string("pages/404").unwrap();
        let response = format!(
            "{status_line}\r\nContent-Length: {}\r\n\r\n{contents}",
            contents.len()
        );
        stream.write_all(response.as_bytes()).unwrap();
        return Ok(String::from_str("Successfully delivered content").unwrap());
    }

    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
    Ok(String::from_str("Successfully delivered content").unwrap())
}

fn get_pages() -> Vec<String> {
    match fs::read_dir("pages") {
        Ok(content) => content
            .map(|file| file.unwrap().file_name().into_string().unwrap())
            .collect(),
        Err(_) => panic!("Pages not found, server unviable"),
    }
}

fn parse_http(request: &str) -> HttpRequest {
    let mut parts = request.split(" ");
    HttpRequest::new(
        String::from_str(parts.next().unwrap()).unwrap(),
        String::from_str(&parts.next().unwrap()[1..]).unwrap(),
        String::from_str(parts.next().unwrap()).unwrap(),
    )
}

struct HttpRequest {
    method: String,
    path: String,
    version: String,
}

impl HttpRequest {
    fn new(method: String, path: String, version: String) -> HttpRequest {
        HttpRequest {
            method: method,
            path: path,
            version: version,
        }
    }
}
