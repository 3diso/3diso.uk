use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    str::FromStr,
    thread,
    time::Duration,
};

fn main() {
    let listener = TcpListener::bind("[::]:31233").unwrap();

    //let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        thread::spawn(|| {
            println!("{}",handle_connection(stream));
        });
    }
}

fn handle_connection(mut stream: TcpStream) -> String {
    println!("Appeared");
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next();

    let content;
    match request_line {
        Some(contents) => content = contents,
        None => return String::from_str("Fail").unwrap(),
    };

    let result;
    match content {
        Ok(contents) => result = contents,
        Err(err) => return String::from_str("Fail").unwrap(),
    };

    let (status_line, filename) = match &result[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
    String::from_str("Success").unwrap()
}
