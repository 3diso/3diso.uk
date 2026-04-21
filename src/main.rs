use std::{
    fs,
    io::{BufReader, Error, prelude::*},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
};

fn main() {
    let listener = TcpListener::bind("[::]:443").unwrap();
    //let listener = TcpListener::bind("127.0.0.1:7878").unwrap(); // for testing purposes

    let valid_pages = Arc::new(get_pages());

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let pages_ref = Arc::clone(&valid_pages);
        thread::spawn(move || {
            match handle_connection(stream, pages_ref){
                Ok(message) => println!("{message}"),
                Err(error) => println!("{error}"),
            }
        });
    }
}

fn handle_connection(mut stream: TcpStream, pages: Arc<Vec<String>>) -> Result<String, Error> {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader
        .lines()
        .next()
        .ok_or(Error::other("no lines"))??;

    let req = parse_http(&request_line[..]);

    println!("Requset received: {request_line}");

    let mut status_line = "HTTP/1.1 200 OK";
    let contents;

    if pages.contains(&req.path) {
        contents = fs::read_to_string(format!("pages/{}", req.path)).unwrap();
    } else if req.path.is_empty() {
        contents = fs::read_to_string("pages/home").unwrap();
    } else if req.path == "resources/styles.css"{
        contents = fs::read_to_string("resources/styles.css").unwrap();
    } else {
        status_line = "HTTP/1.1 404 NOT FOUND";
        contents = fs::read_to_string("pages/404").unwrap();
    }

    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
    Ok(String::from("Successfully delivered content"))
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
    let mut parts = request.split(' ');
    HttpRequest::new(
        String::from(parts.next().unwrap()),
        String::from(&parts.next().unwrap()[1..]),
        String::from(parts.next().unwrap()),
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
            method,
            path,
            version,
        }
    }
}
