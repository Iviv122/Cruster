use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::TcpStream,
};

pub struct Router {}

pub fn handle_connection(folder: String, mut stream: TcpStream) -> () {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let mut req = request_line.split(" ");

    let request_type: &str = req.next().unwrap_or_else(|| "wrong type");
    let mut filename: String = req.next().unwrap_or_else(|| "404.html").to_string();

    let status_line: &str;
    match request_type {
        "GET" => {
            status_line = "HTTP/1.1 200 OK";
        }
        _ => {
            status_line = "HTTP/1.1 405 Method Not Allowed";
            filename = "405.html".to_string();
        }
    };
    if filename.chars().last().unwrap() == '/' {
        filename += "index.html";
    }else{
        filename += ".html"
    }

    let contents = fs::read_to_string(folder + filename.as_str())
        .unwrap_or_else(|_| "No content file somehow".to_string());

    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}

fn uknown_route() -> () {}
