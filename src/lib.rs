use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

/// A simple thread pool implementation.
///
/// `ThreadPool` manages a fixed number of worker threads that execute
/// submitted jobs asynchronously.
///
/// Jobs are sent to workers through a multi-producer, single-consumer (mpsc)
/// channel. Each worker waits for incoming jobs and executes them as they arrive.
pub struct ThreadPool {
    /// Collection of worker threads.
    workers: Vec<Worker>,

    /// Sender side of the job channel.
    ///
    /// Wrapped in `Option` so it can be taken during `Drop`,
    /// allowing workers to exit gracefully.
    sender: Option<mpsc::Sender<Job>>,
}

/// A boxed unit of work to be executed by a worker thread.
///
/// - `FnOnce`: the job can consume captured values
/// - `Send`: can be transferred across threads
/// - `'static`: does not borrow non-static data
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Creates a new `ThreadPool` with the given number of worker threads.
    ///
    /// # Arguments
    ///
    /// * `size` - The number of worker threads in the pool.
    ///
    /// # Panics
    ///
    /// Panics if `size` is `0`, since a thread pool with no threads
    /// cannot execute any jobs.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "ThreadPool size must be greater than zero");

        // Create a channel for sending jobs to workers
        let (sender, receiver) = mpsc::channel();

        // Wrap the receiver so it can be safely shared across threads
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        // Spawn the requested number of worker threads
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Submits a job to the thread pool for execution.
    ///
    /// The job will be picked up by the first available worker thread.
    ///
    /// # Panics
    ///
    /// Panics if the thread pool has already been shut down.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        // Send the job to the worker threads
        self.sender
            .as_ref()
            .expect("ThreadPool has been shut down")
            .send(job)
            .expect("Failed to send job to worker");
    }
}

impl Drop for ThreadPool {
    /// Gracefully shuts down the thread pool.
    ///
    /// Dropping the sender closes the channel, causing all workers
    /// to receive an error from `recv()` and exit their loops.
    ///
    /// The main thread then waits for each worker thread to finish.
    fn drop(&mut self) {
        // Close the channel by dropping the sender
        drop(self.sender.take());

        // Join all worker threads
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().expect("Worker thread panicked");
            }
        }
    }
}

/// A worker thread within the thread pool.
///
/// Each worker waits for jobs on the shared receiver and executes them.
struct Worker {
    /// Identifier for debugging and logging.
    id: usize,

    /// Handle to the underlying thread.
    ///
    /// Wrapped in `Option` so it can be safely taken and joined during shutdown.
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Creates a new worker thread.
    ///
    /// Each worker continuously waits for incoming jobs. When the job channel
    /// is closed, the worker exits its loop and terminates.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                // Lock the receiver and wait for a job
                let message = receiver.lock().unwrap().recv();

                match message {
                    Ok(job) => {
                        println!("Worker {id} received a job; executing.");
                        job();
                    }
                    Err(_) => {
                        // Channel closed — time to shut down
                        println!("Worker {id} disconnected; shutting down.");
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}
