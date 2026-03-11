
use std::{
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
    verbose: bool,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize, verbose: bool) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver),verbose));
        }

        ThreadPool {
            workers: workers,
            sender: Some(sender),
            verbose,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers.drain(..) {
            if self.verbose {
                println!("Shutting down worker {}", worker.id);
            }
            worker.thread.join().unwrap();
        }
    }
}

pub struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>, verbose: bool) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let mess = receiver.lock().unwrap().recv();

                match mess {
                    Ok(job) => {
                        if verbose {
                            println!("Worker {id} got a job; executing");
                        }

                        job();
                    }
                    Err(_) => {
                        if verbose {
                            println!("Worker {id} disconnected; shutting down");
                        }
                        break;
                    }
                }
            }
        });
        Worker { id, thread }
    }
}
