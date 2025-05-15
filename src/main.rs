use std::{ fs, io::{prelude::*, ErrorKind}, net::{TcpListener, TcpStream}, io::BufReader, thread, time::Duration};
const ROOT_FOLDER: &str = "C:/Users/echo-laptop/code/web";//"/var/web";
const IP_ADDR: &str = "127.0.0.1:80";

fn main() {
    let listener = TcpListener::bind(IP_ADDR)
        .expect( format!("Failed to bind listener to {IP_ADDR}").as_str() );

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        stream.set_write_timeout(Some(Duration::from_millis(300)));
        //thread::spawn(|| {
            handle_connection(stream);
        //});
    }
}

fn handle_connection(mut stream: TcpStream) {
    let http_request: HttpMessage = match HttpMessage::from_stream(&stream) {
        Some(res) => res,
        None => { return }
    };

    let response_message = http_request.handle_request();
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

#[derive(Debug)]
struct HttpMessage {
    msg: Vec<u8>,
}

struct HttpMessageBuilder {
    statusline: String,
    headers: Vec<String>,
    data: Vec<u8>,
}

impl HttpMessageBuilder {
    fn status_line(&mut self, str: &str) -> &Self {
        self.statusline = str.to_string();
        self
    }

    fn header(&mut self, str: String) -> &Self {
        self.headers.push(str);
        self
    }

    fn data(&mut self, vec: Vec<u8>) -> &Self {
        self.data = vec;
        self
    }

    fn build(self) -> HttpMessage {
        // status-line\r\n
        // (header \r\n)*
        // \r\n
        // (data)
        let mut msg = Vec::new();

        msg.extend(self.statusline.as_bytes());
        msg.extend("\r\n".bytes());
        
        if !self.headers.is_empty() {
            msg.extend(self.headers.join("\r\n").as_bytes()); 
            msg.extend("\r\n\r\n".bytes());
        }

        if !self.data.is_empty() {
            msg.extend(self.data);
        }
        println!("response: {:?}", String::from_utf8(msg.clone()));
        HttpMessage { msg }
    }
}

impl HttpMessage {
    fn builder() -> HttpMessageBuilder {
        HttpMessageBuilder { statusline: String::new(), headers: Vec::new(), data: Vec::new() }
    }

    fn startline(&self) -> String {
        let _msg = String::from_utf8(self.msg.clone()).unwrap();
        let val= _msg.split("\r\n").nth(0).unwrap();
        val.to_string()
    }

    fn headers(&self) -> String {
        let _msg = String::from_utf8(self.msg.clone()).unwrap();
        let val = _msg.split("\r\n\r\n").nth(0).unwrap();
        val.to_string()
    }

    fn data(&self) -> Vec<u8> {
        // Data is seperated by an string "\r\n\r\n"
        let _msg = String::from_utf8(self.msg.clone()).unwrap();
        let val = _msg.split("\r\n\r\n").nth(1).unwrap();
        val.as_bytes().to_vec()
    }

    /// Return the header value from key name if it exists
    fn get_header(&self, key: &str) -> Option<String> {
        todo!()
    }

    fn method(&self) -> HttpMethod {
        match self.startline().split(" ").collect::<Vec<&str>>()[0].to_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            "TRACE" => HttpMethod::Trace,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "POST" => HttpMethod::Post,
            "PATCH" => HttpMethod::Patch,
            "CONNECT" => HttpMethod::Connect,
            _ => panic!("Invalid HTTP Method requested"),
        }
    }

    fn from_stream(stream: &TcpStream) -> Option<HttpMessage> {
        let reader = BufReader::new(stream);
        let message: Vec<String> =
            reader
            .lines()
            .map(|result| match result {
                Ok(a) => a,
                Err(_) => String::new(),
            })
            .take_while(|line| !line.is_empty())
            .collect();
        let msg = message.join("\r\n");
        println!("request: {:?}", msg);
        match validate_incoming_msg(msg.clone()) {
            true =>  Some ( HttpMessage { msg: msg.into_bytes() } ),
            false => None, 
        }
    }

    fn handle_request(&self) -> HttpMessage {
        use HttpMethod::*;
        match self.method() {
            Get => {
                let binding = self.startline();
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

    fn finalize(&self) -> &[u8] {
        self.msg.as_slice()
    } 
}

fn split_msg(msg: Vec<u8>) -> Vec<String> {
    todo!()
}

fn validate_incoming_msg(msg: String) -> bool {
    let val= msg.split("\r\n").nth(0).unwrap();
    match val.split(" ").collect::<Vec<&str>>()[2] {
        "HTTP/1.0" | "HTTP/1.1" => true,
        _ => false,
    }
}

enum HttpMethod {
    Get,
    Head,
    Options,
    Trace,
    Put,
    Delete,
    Post,
    Patch,
    Connect,
}