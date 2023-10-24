use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
};

use nom::AsBytes;

const OK_RESPONSE: &[u8; 19] = b"HTTP/1.1 200 OK\r\n\r\n";
const NOT_FOUND_RESPONSE: &[u8; 26] = b"HTTP/1.1 404 Not Found\r\n\r\n";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let request_buffer = BufReader::new(&stream);

                let http_request: Vec<_> = request_buffer
                    .lines()
                    .map(|line| line.unwrap())
                    .take_while(|line| !line.is_empty())
                    .collect();

                let path = get_path(&http_request).unwrap();

                if path == "/" {
                    stream.write(OK_RESPONSE).unwrap();
                } else if path.starts_with("/echo") {
                    let input = match path.split("/").nth(2) {
                        Some(input) => input,
                        None => panic!("Failed to get input from path"),
                    };
                    stream.write_all(ok_response(input).as_bytes()).unwrap();
                } else {
                    stream.write_all(NOT_FOUND_RESPONSE).unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn get_path(http_request: &Vec<String>) -> Option<String> {
    return match http_request.first() {
        Some(first_line) => {
            let first_line_splitted = first_line.split_ascii_whitespace().collect::<Vec<_>>();

            let maybe_path = first_line_splitted.get(1);

            match maybe_path.map(|s| s.to_string()) {
                Some(path) => Some(path),
                None => None,
            }
        }
        None => {
            panic!("Wrongly formatted http request")
        }
    };
}

fn ok_response(body: &str) -> Vec<u8> {
    let mut response = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n".to_vec();
    let content_length = body.len();

    let content_length_header = format!("Content-Length: {content_length}\r\n\r\n");
    response.extend_from_slice(content_length_header.as_bytes());

    response.extend_from_slice(body.as_bytes());

    response
}
