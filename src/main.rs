use std::{ io::{prelude::*, BufReader, ErrorKind}, net::{TcpListener, TcpStream}, fs, thread};
const ROOT_FOLDER: &str = "/var/web";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:80").unwrap();
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        thread::spawn(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let http_request: Vec<String> = buf_reader
        .lines()
        .map(|result| match result {
            Ok(a) => a,
            Err(_) => String::new(),
        })
        .take_while(|line| !line.is_empty())
        .collect();

    println!("{:?}", http_request);
    let requested_file = if !http_request.is_empty() && http_request[0].contains("GET") {
        http_request[0].rsplit(' ').nth(1).unwrap()
    } else { return };

    let file = if requested_file == "/" {
        "/index.html"
    } else {
        requested_file
    };

    let err_404_page = format!("{ROOT_FOLDER}/404.html");
    let path = format!("{ROOT_FOLDER}{file}");
    
    let (status_line, data) = match fs::read(&path) {
        Ok(data) => { 
            ( "HTTP/1.1 200 OK\r\n", data )
        },
        Err(err) => {
            eprintln!("Error opening {path}: {err}");
            match err.kind() {
                ErrorKind::NotFound => { 
                    if file.ends_with(".html") {
                        ( "HTTP/1.1 404 NOT FOUND\r\n", fs::read(err_404_page).unwrap() )
                    } else {
                        ( "HTTP/1.1 404 NOT FOUND\r\n", Vec::new() ) 
                    }
                },
                _ => {

                    ( "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n", Vec::new() )
                }
            }
        },  
    };

    let length = data.len();
    let mut response: Vec<u8> = status_line.into();

    if length > 0 {
        let mut contents: Vec<u8> = format!("Content-Length: {length}\r\n\r\n").into();
        contents.extend(data);
        response.extend(contents);
    }

    stream.write_all(response.as_slice()).unwrap();
}