use std::{
    env, fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::{
        LazyLock, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use Cruster::ThreadPool;

static HOST: &str = "127.0.0.1:";
static PORT: LazyLock<Mutex<u16>> = LazyLock::new(|| Mutex::new(8080));
static THREAD_COUNT: LazyLock<Mutex<usize>> = LazyLock::new(|| Mutex::new(5));
static VISITS: AtomicUsize = AtomicUsize::new(0);
static VERBOSE: AtomicBool = AtomicBool::new(false);

fn main() {
    if let Err(err) = process_args() {
        println!("cruster: {}", err);
        println!("");
        show_usage();
        return;
    }

    let adress: String = HOST.to_owned() + PORT.lock().unwrap().to_string().as_str();
    let threads = THREAD_COUNT.lock().unwrap();

    if VERBOSE.load(Ordering::Relaxed) {
        println!("Occupied adress: {}", adress);
        println!("Occupied threads: {}", threads);
    }

    let listener = TcpListener::bind(adress).unwrap();
    let tpool = ThreadPool::new(*threads,VERBOSE.load(Ordering::Relaxed));

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        tpool.execute(|| {
            handle_connection(stream);
        });
        if VERBOSE.load(Ordering::Relaxed) {
            println!(
                "Visits: {}",
                VISITS.fetch_add(1, Ordering::AcqRel)
            );
        }
    }

    println!("Shutting down")
}

fn show_usage() -> () {
    println!("usage: cruster [OPTION]");
    println!("p port_number (standart: 8080, or any free)");
    println!("t threads_used (standart: 5)");
    println!("v verbose (standart: false)")
}

fn process_args() -> Result<(), String> {
    let mut args = env::args().skip(1);

    while let Some(flag) = args.next() {
        match flag.as_str() {
            "p" | "t" => {
                let value = args
                    .next()
                    .ok_or(format!("Expected value after: {}", flag))?;

                let num: u16 = value
                    .parse()
                    .map_err(|_| format!("Invalid number: {}", value))?;

                match flag.as_str() {
                    "p" => {
                        let mut port = PORT.lock().unwrap();
                        *port = num;
                    }
                    "t" => {
                        let mut thread = THREAD_COUNT.lock().unwrap();
                        *thread = usize::from(num);
                    }
                    _ => {}
                }
            }
            "v" => {
                VERBOSE.store(true, std::sync::atomic::Ordering::Relaxed);
            }

            _ => {
                return Err(format!("Unknown flag {}", flag));
            }
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> () {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = if request_line == "GET / HTTP/1.1" {
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}
