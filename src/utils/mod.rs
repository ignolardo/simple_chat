use std::sync::Arc;
use tokio::sync::Mutex;

pub mod thread_pool;
pub mod channels;

pub type ArcMut<T> = Arc<Mutex<T>>;

pub fn new_arcmut<T>(value: T) -> ArcMut<T> {
      Arc::new(Mutex::new(value))
}



pub type Tx<T> = tokio::sync::mpsc::UnboundedSender<T>;
pub type Rx<T> = tokio::sync::mpsc::UnboundedReceiver<T>;