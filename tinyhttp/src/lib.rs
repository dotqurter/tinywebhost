use std::{net::TcpStream, io::{BufReader, BufRead}};
pub enum HttpMethod {
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

pub struct HttpRequest {
    pub start_line: String,
    pub headers: Vec<String>,
    pub body: Option<Box<[u8]>>,
}
#[derive(Debug)]
pub enum HttpMethodErr {
    InvalidMethod,
}
impl HttpRequest {
    pub fn method(&self) -> Result<HttpMethod, HttpMethodErr> {
        match self.start_line.split(" ").nth(0).unwrap().to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "HEAD" => Ok(HttpMethod::Head),
            "OPTIONS" => Ok(HttpMethod::Options),
            "TRACE" => Ok(HttpMethod::Trace),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "POST" => Ok(HttpMethod::Post),
            "PATCH" => Ok(HttpMethod::Patch),
            "CONNECT" => Ok(HttpMethod::Connect),
            _ => Err(HttpMethodErr::InvalidMethod),
        }
    }
    pub fn from_stream(stream: &TcpStream) -> Option<HttpRequest> {
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
        match validate_incoming_msg(&msg) {
            true =>  Some ( HttpRequest { start_line: startline(&msg), headers: headers(&msg), body: body(&msg) } ),
            false => None, 
        }
    }
}

pub struct HttpResponse {
    pub status_line: String,
    pub headers: Vec<String>,
    pub body: Option<Box<[u8]>>,
}
impl HttpResponse {
    pub fn get_header(&self, key: impl AsRef<str>) -> Option<String> {
        let t = key.as_ref().to_uppercase();
        for line in &self.headers {
            let line = line.to_uppercase();
            if line.contains(&t) {
                let val = line.split_once(":").unwrap().1
                    .trim()
                    .to_string();
                return Some (val); 
            }
        }
        None
    }
    pub fn build(&self) -> Vec<u8> {
        // status-line\r\n
        // (header \r\n)*
        // \r\n
        // (data)
        let mut msg = Vec::new();

        msg.extend(self.status_line.as_bytes());
        msg.extend("\r\n".bytes());
        
        if !self.headers.is_empty() {
            msg.extend(self.headers.join("\r\n").as_bytes()); 
            msg.extend("\r\n\r\n".bytes());
        }

        if self.body.is_some() {
            let body = self.body.clone().unwrap();
            msg.extend(body.into_vec());
        }
        //println!("response: {:?}", String::from_utf8(msg.clone()));
        msg
    }
}

pub struct HttpResponseBuilder {
    response: HttpResponse,
}
impl HttpResponseBuilder {
    pub fn new() -> Self {
        Self { response: HttpResponse { status_line: String::new(), headers: Vec::new(), body: None }}
    }
    pub fn status_line(&mut self, str: impl Into<String>) -> &Self {
        self.response.status_line = str.into();
        self
    }

    pub fn header(&mut self, str: impl Into<String>) -> &Self {
        self.response.headers.push(str.into());
        self
    }

    pub fn body(&mut self, vec: impl Into<Box<[u8]>>) -> &Self {
        self.response.body = Some(vec.into());
        self
    }

    pub fn build(self) -> HttpResponse {
        self.response
    }
} 

fn headers(msg: impl AsRef<str>) -> Vec<String> {
    let _msg = msg.as_ref();
    let binding = _msg.split("\r\n\r\n").nth(0).unwrap();

    let mut vec = Vec::new();
    let val = binding.split("\r\n");
    for i in val {
        vec.push(i.to_string());
    }
    vec
}

fn startline(msg: impl AsRef<str>) -> String {
    let _msg = msg.as_ref();
    let val = _msg.split("\r\n").nth(0).unwrap();
    val.to_string()
}

fn body(msg: impl AsRef<str>) -> Option<Box<[u8]>> {
    let _msg = msg.as_ref();
    let val = _msg.split("\r\n\r\n").nth(1);
    match val.is_some() {
        true => return Some (val.unwrap().as_bytes().into()),
        false => return None
    }
}

fn validate_incoming_msg(msg: impl AsRef<str>) -> bool {
    let b = msg.as_ref().to_uppercase();
    let val = b.split("\r\n").nth(0).unwrap();
    match val.split(" ").nth(2).unwrap() {
        "HTTP/1.0" | "HTTP/1.1" => true,
        _ => false,
    }
}
