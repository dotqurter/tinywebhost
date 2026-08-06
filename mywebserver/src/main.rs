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

use std::path::Path;
fn handle_request(message: HttpRequest) -> HttpResponse {
    let mut builder = HttpResponseBuilder::new();

    let root_folder = Path::new(root_folder())
        .canonicalize().expect("Couldn't canonicalize the root folder");

    match message.method().unwrap() {
        Get => {
            let binding = message.start_line;
            let requested_file = binding.split(' ').nth(1).unwrap();

            let mut path = root_folder.to_path_buf();

            // clean the leading forwardslash
            let file = match requested_file {
                "/" => "index.html",
                _ => requested_file.get(1..).unwrap(),
            };

            path.push(file);
            let path = match path.canonicalize() {
                Ok(path) => path,
                Err(_) => {
                    eprintln!("Couldn't canonicalize path: {path:?}");
                    builder.status_line("HTTP/1.1 404 Not Found");
                    return builder.build();
                }
            };

            // Avoid path traversal issues
            if !path.starts_with(&root_folder) {
                eprintln!("Attempted to traverse outside path, looking for file {path:?}");
                builder.status_line("HTTP/1.1 400 Bad Request");
                return builder.build();
            }

            match fs::read(&path) {
                Ok(data) => {
                    let length = data.len();
                    builder.status_line("HTTP/1.1 200 OK");
                    builder.header(format!("Content-Length: {length}"));
                    builder.header(format!("Server: mywebserver"));
                    builder.body(data);
                },
                Err(err) => {
                    eprintln!("Error opening {path:?}: {err}");
                    match err.kind() {
                        ErrorKind::NotFound => {
                            builder.status_line("HTTP/1.1 404 Not Found");
                            if let Ok(errpage) = fs::read(&root_folder.with_file_name("404.html")) {
                                let length = errpage.len();
                                builder.header(format!("Content-Length: {length}"));
                                builder.body(errpage);
                            }
                        },
                        _ => {
                            eprintln!("Unhandled error: {}", err);
                            builder.status_line("HTTP/1.1 501 Not Implemented");
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
