use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;
//use std::future::Future;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ThreadPool {
      workers: Vec<Worker>,
      sender: Option<Sender<Job>>,
}

//type Job = Box<dyn Future<Output = ()> + Send + 'static>;
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
      pub fn new(size: usize) -> ThreadPool {
          assert!(size > 0);
  
          let (sender, receiver) = mpsc::channel(size);
  
          let receiver = Arc::new(Mutex::new(receiver));
  
          let mut workers = Vec::with_capacity(size);
  
          for id in 0..size {
              workers.push(Worker::new(id, Arc::clone(&receiver)));
          }
  
          ThreadPool {
              workers,
              sender: Some(sender),
          }
      }
  
      pub async fn execute<F>(&self, f: F)
      where
            //F::Output: Send + 'static,
            //F: Future<Output = ()> + Send + 'static,
            F: FnOnce() + Send + 'static
      {
          let job = Box::new(f);
  
          self.sender.as_ref().unwrap().send(job).await.unwrap();
      }
  }
  
  impl Drop for ThreadPool {
      fn drop(&mut self) {
          drop(self.sender.take());
  
          for worker in &mut self.workers {
              if let Some(thread) = worker.thread.take() {
                  thread.abort();
              }
          }
      }
  }
  
  struct Worker {
      _id: usize,
      thread: Option<JoinHandle<()>>,
  }
  
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<Receiver<Job>>>) -> Worker {
        let thread = tokio::spawn(async move {
            loop {
                let result = receiver.lock().await.recv().await;
    
                match result {
                    Some(job) => {
                        job();
                    }
                    None => {
                        break;
                    }
                }
            }
        });
  
        Worker {
            _id: id,
            thread: Some(thread),
        }
    }

}