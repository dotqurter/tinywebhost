use std::{ io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, fs, };
static ROOT_FOLDER: &str = "C:/Users/echo/code/website";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<String> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let file = http_request[0]
        .strip_prefix("GET ").unwrap()
        .strip_suffix(" HTTP/1.1").unwrap();
        
    let path = format!("{ROOT_FOLDER}{file}");

    let (status_line, data) = match fs::read_to_string(&path) {
        Ok(data) => { 
            ( "HTTP/1.1 200 OK".to_string(), data )
        },
        _ => { 
            println!("Connection requested nonexistent file, {}", path); 
            ( "HTTP/1.1 404 NOT FOUND".to_string(), "".to_string() ) 
        },
    };

    let length = data.len();
    let mut contents = "".to_string();

    if length > 0 {
        contents = format!("Content-Length: {length}\r\n\r\n{data}");
    }

    let response = format!("{status_line}\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}