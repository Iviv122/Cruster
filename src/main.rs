use std::{
    env,
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, TcpListener},
    sync::{
        LazyLock, Mutex,
        atomic::{AtomicBool, AtomicU16, AtomicUsize, Ordering},
    },
};

use cruster::{router::handle_connection, thread_pool::ThreadPool};

static HOST: &str = "0.0.0.0";
static THREAD_COUNT: AtomicUsize = AtomicUsize::new(5);
static PORT: AtomicU16 = AtomicU16::new(8080);
static VISITS: AtomicUsize = AtomicUsize::new(0);
static VERBOSE: AtomicBool = AtomicBool::new(false);

static FOLDER: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new("public".to_string()));

fn main() {
    if let Err(err) = process_args() {
        println!("cruster: {}", err);
        println!("");
        show_usage();
        return;
    }

    port_verification();

    let adress: String = format!("{}:{}", HOST, PORT.load(Ordering::Relaxed));
    let threads = THREAD_COUNT.load(Ordering::Relaxed);

    println!("Occupied adress: {}", adress);
    println!("Occupied threads: {}", threads);

    let addr = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, PORT.load(Ordering::Relaxed), 0, 0);
    let listener = TcpListener::bind(addr).unwrap();
    let tpool = ThreadPool::new(threads, VERBOSE.load(Ordering::Relaxed));

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        tpool.execute(|| {
            handle_connection(FOLDER.lock().unwrap().to_string(), stream);
        });
        if VERBOSE.load(Ordering::Relaxed) {
            println!("Visits: {}", VISITS.fetch_add(1, Ordering::AcqRel));
        }
    }

    if VERBOSE.load(Ordering::Relaxed) {
        println!("Shutting down")
    }
}

fn show_usage() -> () {
    println!("usage: cruster [OPTION]");
    println!("p port_number (standart: 8080, or any free)");
    println!("t threads_used (standart: 5)");
    println!("v verbose (standart: false)");
    println!("f folder (standart: 'public')");
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
                        PORT.store(num, Ordering::Relaxed);
                    }
                    "t" => {
                        THREAD_COUNT.store(usize::from(num), Ordering::Relaxed);
                    }

                    _ => {}
                }
            }
            "f" => {
                let value = args
                    .next()
                    .ok_or(format!("Expected folder name: {}", flag))?;

                let mut folder = FOLDER.lock().unwrap();
                *folder = value;                    
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

fn port_verification() {
    while !is_port_free() {
        let num = PORT.load(Ordering::Relaxed);
        if VERBOSE.load(Ordering::Relaxed) {
            println!("Port {} was occupied, trying next one", num);
        }
        PORT.store(num + 1, Ordering::Relaxed);
    }
}

fn is_port_free() -> bool {
    let addr = SocketAddrV6::new(Ipv6Addr::LOCALHOST, PORT.load(Ordering::Relaxed), 0, 0);
    TcpListener::bind(addr).is_ok()
}