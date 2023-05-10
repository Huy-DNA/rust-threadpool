use std::{
    thread,
    thread::JoinHandle,
    fmt,
    sync::{Arc, Mutex, mpsc},
    panic,
};

/// An error struct specifying that a non-positive value has been
/// passed to a caller
///
/// ```rust,should_panic
/// # use threadpool::ThreadPool;
/// fn main() {
///     ThreadPool::build(0).unwrap();
/// } 
/// ```
#[derive(Debug, Clone)]
pub struct ThreadCountError {
    caller: &'static str,
    invalid_val: usize,
}

impl fmt::Display for ThreadCountError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: Invalid thread count: {}. Thread count must be a positive number.", self.caller, self.invalid_val)
    }
}

/// A struct representing a job that can sent to other threads
/// and can only be called once
pub type Job = Box<dyn FnOnce() + Send + 'static>;

/// A struct representing a worker holding a thread for executing job
pub struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    /// Returns a new worker with its thread initiated
    ///
    /// # Arguments
    ///
    /// * `id` - The worker's ID
    ///
    /// * `receiver` - A lock-protected receiver shared with other workers
    /// within the same thread pool
    ///
    /// # Caution
    ///
    /// If a worker panics, other workers within the pool will panic 
    /// due to mutex poisoning and effectively the thread pool is dead.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        Worker {
            id,
            thread: Some(thread::spawn(move || {
                eprintln!("Thread {} is starting up", id);
                loop {
                    match receiver.lock().unwrap().recv() {
                        Ok(job) => job(),
                        Err(_) => {
                            eprintln!("Thread {} is shutting down", id);
                            break;
                        }
                    }
                }
            })),
        }
    }
}

/// A struct representing a thread pool
pub struct ThreadPool {
    workers: Vec<Worker>, 
    job_sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    /// Returns a `Result<ThreadPool, ThreadCountError>`
    ///
    /// # Arguments
    ///
    /// * `thread_count` - The number of threads in the pool,
    /// if `thread_count < 0`, returns an `Err`
    ///
    /// # Examples
    ///
    /// This code panics
    ///
    /// ```rust,should_panic
    /// # use threadpool::ThreadPool;
    /// fn main() {
    ///     ThreadPool::build(0).unwrap();
    /// }
    ///
    /// Negative `thread_count` passed to `build` would not compile.
    ///
    /// ```rust,compile_fail
    /// # use threadpool::ThreadPool;
    /// fn main() {
    ///     ThreadPool::build(-1);
    /// }
    /// ```
    /// 
    pub fn build(thread_count: usize) -> Result<ThreadPool, ThreadCountError> {
        if thread_count <= 0 {
            return Err(ThreadCountError { caller: "ThreadPool::new()", invalid_val: thread_count }); 
        }
        
        let (job_sender, job_receiver) = mpsc::channel();
        let job_sender = Some(job_sender);
        let job_receiver = Arc::new(Mutex::new(job_receiver));

        let mut workers = Vec::with_capacity(thread_count);
        
        for id in 0..thread_count {
            workers.push(Worker::new(id, job_receiver.clone()));
        }

        Ok(ThreadPool{ workers, job_sender })
    }
   
    /// Send a job to the thread pool to execute it
    ///
    /// # Arguments
    ///
    /// * `&mut self`
    ///
    /// * `job` - A callable implementing `FnOnce() + Send + 'static`
    ///
    /// # Caution
    ///
    /// Careful to guarantee that the callable can not panic or else
    /// the thread pool can possibly be "dead" and will silently stop
    /// executing job
    ///
    /// However, this can be detected if a panic is observed when the 
    /// thread pool is dropped
    /// by panicking
    pub fn execute<F>(&mut self, job: F) 
        where F: FnOnce() + Send + 'static {
        self.job_sender.as_ref().unwrap()
                       .send(Box::new(job)).unwrap(); 
    }
}

impl Drop for ThreadPool {
    /// Gracefully shutdown the thread pool
    ///
    /// Drop the job sender and wait for all threads to shutdown
    /// 
    /// # Panics
    ///
    /// If one of the worker had panicked and thus, terminated prematurely,
    /// this method panics
    ///
    /// ```rust,should_panic
    /// # use threadpool::ThreadPool;
    /// fn main() {
    ///     let mut pool = ThreadPool::build(10).unwrap();
    ///     pool.execute(|| panic!("Error"));
    /// }
    /// ```
    fn drop(&mut self) {
        drop(self.job_sender.take().unwrap());
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().expect("Warning: Some workers seem to have panicked. \
                                      This likely led to wrong behavior");
            }
        }
    }
}
