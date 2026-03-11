use std::{
    fs::{self, canonicalize},
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    ops::Add,
    path::{Path},
};

pub struct Router {}

pub fn handle_connection(folder: String, mut stream: TcpStream) -> () {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let mut req = request_line.split(" ");

    let request_type: &str = req.next().unwrap_or_else(|| "wrong type");
    let mut filename: String = req.next().unwrap_or_else(|| "404").to_string();

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
    } else if has_extension(filename.as_str()) {
    } else {
        filename += ".html"
    }

    let homedir = fs::canonicalize(folder.clone().as_str()).unwrap();
    let homedir_path: &str = homedir.to_str().unwrap();

    let can: Result<std::path::PathBuf, std::io::Error> = canonicalize(folder.clone().add(filename.as_str()));
    let binding = can.unwrap_or_else(|_| homedir.clone());
    let can_s = binding.to_str().unwrap();
    println!("{}",can_s);

    let diff = can_s.strip_prefix(homedir_path);

    if diff.is_none_or(|s| s.trim().is_empty()) {
        // bad file or doesn't exist

        let ff = format!("{folder}/404.html");
        let contents =fs::read_to_string(ff).unwrap_or_else(|_| "No /404.html".to_string());
        let length = contents.len();
        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).unwrap();
    } else {
        // everything is ok

        let contents = fs::read_to_string(can_s).unwrap_or_else(|_| "No /404.html".to_string());

        let length = contents.len();
        let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
        stream.write_all(response.as_bytes()).unwrap();
    }
}

fn has_extension(filename: &str) -> bool {
    !Path::new(filename).extension().is_none()
}
