#![allow(unused, private_interfaces)]
use std::{ net::{TcpStream}, io::BufReader, io::BufRead };

#[derive(Debug)]
pub struct HttpMessage {
    msg: Vec<u8>,
}

pub struct HttpMessageBuilder {
    statusline: String,
    headers: Vec<String>,
    data: Vec<u8>,
}

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

impl HttpMessageBuilder {
    pub fn status_line(&mut self, str: &str) -> &Self {
        self.statusline = str.to_string();
        self
    }

    pub fn header(&mut self, str: String) -> &Self {
        self.headers.push(str);
        self
    }

    pub fn data(&mut self, vec: Vec<u8>) -> &Self {
        self.data = vec;
        self
    }

    pub fn build(self) -> HttpMessage {
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
        //println!("response: {:?}", String::from_utf8(msg.clone()));
        HttpMessage { msg }
    }
}

impl HttpMessage {
    pub fn builder() -> HttpMessageBuilder {
        HttpMessageBuilder { statusline: String::new(), headers: Vec::new(), data: Vec::new() }
    }

    pub fn startline(&self) -> String {
        let _msg = String::from_utf8(self.msg.clone()).unwrap();
        let val= _msg.split("\r\n").nth(0).unwrap();
        val.to_string()
    }

    pub fn headers(&self) -> String {
        let _msg = String::from_utf8(self.msg.clone()).unwrap();
        let binding = _msg.split("\r\n\r\n").nth(0).unwrap();

        let val = binding.split_once("\r\n").unwrap().1;
        val.to_string()
    }

    pub fn data(&self) -> Vec<u8> {
        // Data is seperated by an string "\r\n\r\n"
        let _msg = String::from_utf8(self.msg.clone()).unwrap();
        let val = _msg.split("\r\n\r\n").nth(1).unwrap();
        val.as_bytes().to_vec()
    }

    /// Return the header value from key name if it exists
    pub fn get_header(&self, key: &str) -> Option<String> {
        let t = key.to_uppercase();
        for line in self.headers().to_uppercase().lines() {
            if line.contains(t.as_str()) {
                let val = line.split_once(":").unwrap().1
                    .trim().to_string();
                return Some (val); 
            }
        }
        None
    }

    pub fn method(&self) -> HttpMethod {
        match self.startline().split(" ").nth(0).unwrap().to_uppercase().as_str() {
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

    pub fn from_stream(stream: &TcpStream) -> Option<HttpMessage> {
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

    pub fn finalize(&self) -> &[u8] {
        self.msg.as_slice()
    } 
}


fn validate_incoming_msg(msg: String) -> bool {
    let b = msg.to_uppercase();
    let val= b.split("\r\n").nth(0).unwrap();
    match val.split(" ").nth(2).unwrap() {
        "HTTP/1.0" | "HTTP/1.1" => true,
        _ => false,
    }
}