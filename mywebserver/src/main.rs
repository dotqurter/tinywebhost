use std::{ fs, io::{prelude::*, ErrorKind}, net::{TcpListener, TcpStream}, thread};
use tinyhttpparser::{ HttpMethod, HttpMessage };
const ROOT_FOLDER: &str = "/var/web";
const IP_ADDR: &str = "127.0.0.1:80";

fn main() {
    let listener = TcpListener::bind(IP_ADDR)
        .expect( format!("Failed to bind listener to {IP_ADDR}").as_str() );

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let http_request: HttpMessage = match HttpMessage::from_stream(&stream) {
        Some(res) => res,
        None => { return }
    };
    //Example of using get_header
    // {
    //     let header_test = "Accept";
    //     println!("Test == {header_test}: {}", http_request.get_header(header_test).unwrap());
    // }
    let response_message = handle_request(http_request);
    stream.write_all(response_message.finalize()).unwrap();
}

fn handle_get_request(file: &str) -> HttpMessage {
    let path = format!("{ROOT_FOLDER}{file}");
    let mut message = HttpMessage::builder();
    match fs::read(&path) {
        Ok(data) => {
            let length = data.len();
            message.status_line("HTTP/1.1 200 OK");
            message.header(format!("Content-Length: {length}"));
            message.header(format!("Server: mywebserver"));
            message.data(data);
        },
        Err(err) => {
            eprintln!("Error opening {path}: {err}");
            match err.kind() {
                ErrorKind::NotFound => {
                    message.status_line("HTTP/1.1 404 NOT FOUND");
                    match fs::read( format!("{ROOT_FOLDER}/404.html")) {
                        Ok(data) => {
                            let length = data.len();
                            message.header(format!("Content-Length: {length}"));
                            message.data(data);
                        },
                        _ => {},
                    }
                },
                _ => {
                    eprintln!("Unhandled error: {}", err);
                    message.status_line("HTTP/1.1 500 INTERNAL SERVER ERROR");
                }
            }
        },  
    }
    message.build()
}

fn handle_request(message: HttpMessage) -> HttpMessage {
    use HttpMethod::*;
    match message.method() {
        Get => {
            let binding = message.startline();
            let requested_file = binding.split(' ').nth(1).unwrap();

            let file = if requested_file == "/" {
                "/index.html"
            } else {
                requested_file
            };

            handle_get_request(file)
        },
        _ => {
            let mut msg = HttpMessage::builder();
            msg.status_line("HTTP/1.1 405 Method Not Allowed");
            msg.build()
        },
    }
}