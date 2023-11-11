use std::{
    sync::{
        mpsc::{Receiver, Sender},
        *,
    },
    thread::{self, JoinHandle},
};

use itertools::Itertools;

pub struct ThreadPool {
    threads: Vec<Worker>,
    sender: Sender<Job>,
}

pub type Job = Box<dyn FnOnce() -> () + Send + 'static>;

impl ThreadPool {
    pub fn new(threads: usize) -> ThreadPool {
        let (sender, receiver_raw) = mpsc::channel::<Job>();
        let receiver = Arc::new(Mutex::new(receiver_raw));
        let threads = (0..threads)
            .map(|id| Worker::new(id, Arc::clone(&receiver)))
            .collect_vec();
        ThreadPool { threads, sender }
    }

    pub fn run(&self, job: Job) -> Result<(), mpsc::SendError<Job>> {
        self.sender.send(job)
    }
}

struct Worker {
    id: usize,
    thread_handle: JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread_handle = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();

            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");
                    job()
                }
                Err(_) => {
                    println!("Worker {id} FAILED to got a job");
                    break;
                }
            };
        });
        Worker { id, thread_handle }
    }
}
