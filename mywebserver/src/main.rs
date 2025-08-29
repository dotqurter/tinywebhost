use std::{ env, fs, io::{prelude::*, ErrorKind}, net::TcpListener, thread, sync::OnceLock };
use tinyhttp::{ HttpMethod::*, HttpRequest, HttpResponse, HttpResponseBuilder };

fn root_folder() -> &'static str {
    static ROOT_FOLDER: OnceLock<String> = OnceLock::new();
    ROOT_FOLDER.get_or_init(|| env::var("ROOT_FOLDER").unwrap_or(r"/var/web".to_owned()))
}
fn ip_addr() -> &'static str {
    static IP_ADDR: OnceLock<String> = OnceLock::new();
    IP_ADDR.get_or_init(|| env::var("IP_ADDR").unwrap_or(r"0.0.0.0:80".to_owned()))
}

fn main() {
    let ip_addr = ip_addr();
    let listener = TcpListener::bind(&ip_addr)
        .expect( format!("Failed to bind listener to {ip_addr}").as_str() );

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };
        thread::spawn(move || {
            let http_request: HttpRequest = match HttpRequest::from_stream(&stream) {
                Some(res) => res,
                None => { return }
            };
            
            let response_message = handle_request(http_request);
            stream.write_all(response_message.build().as_slice()).unwrap();
        });
    }
}

fn handle_request(message: HttpRequest) -> HttpResponse {
    let mut builder = HttpResponseBuilder::new();
    let root_folder = root_folder();
    match message.method().unwrap() {
        Get => {
            let binding = message.start_line;
            let requested_file = binding.split(' ').nth(1).unwrap();

            let file = match requested_file {
                "/" => "/index.html",
                _ => requested_file,
            };

            let path = format!("{root_folder}/{file}");
            match fs::read(&path) {
                Ok(data) => {
                    let length = data.len();
                    builder.status_line("HTTP/1.1 200 OK");
                    builder.header(format!("Content-Length: {length}"));
                    builder.header(format!("Server: mywebserver"));
                    builder.body(data);
                },
                Err(err) => {
                    eprintln!("Error opening {path}: {err}");
                    match err.kind() {
                        ErrorKind::NotFound => {
                            builder.status_line("HTTP/1.1 404 NOT FOUND");
                            match fs::read( format!("{root_folder}/404.html")) {
                                Ok(data) => {
                                    let length = data.len();
                                    builder.header(format!("Content-Length: {length}"));
                                    builder.body(data);
                                },
                                _ => {},
                            }
                        },
                        _ => {
                            eprintln!("Unhandled error: {}", err);
                            builder.status_line("HTTP/1.1 500 INTERNAL SERVER ERROR");
                        }
                    }
                },  
            }
        },
        _ => {
            builder.status_line("HTTP/1.1 405 Method Not Allowed");
        },
    }
    builder.build()
}