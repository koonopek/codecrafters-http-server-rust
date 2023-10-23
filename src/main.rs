use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
};

const ok_response: &[u8; 19] = b"HTTP/1.1 200 OK\r\n\r\n";
const not_found_response: &[u8; 26] = b"HTTP/1.1 404 Not Found\r\n\r\n";

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
                    stream.write_all(ok_response).unwrap();
                } else {
                    stream.write_all(not_found_response).unwrap();
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
