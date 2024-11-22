use tokio::sync::mpsc::{self, error::SendError};

pub struct Stream<T,U>(pub mpsc::Sender<T>, pub mpsc::Receiver<U>);

//pub struct DoubleChannel<T,U>(Stream<T,U>,Stream<U,T>);

/* impl<T,U> DoubleChannel<T,U> {
      pub fn new(size: usize) -> Self {
            let (tx1, rx1) = mpsc::channel::<T>(size);
            let (tx2, rx2) = mpsc::channel::<U>(size);
            Self(Stream(tx1, rx2), Stream(tx2,rx1))
      }
} */

pub fn double_channel<T,U>(size: usize) -> (Stream<T,U>,Stream<U,T>) {
      let (tx1, rx1) = mpsc::channel::<T>(size);
      let (tx2, rx2) = mpsc::channel::<U>(size);
      (Stream(tx1,rx2), Stream(tx2,rx1))
}

impl<T,U> Stream<T,U> {

      pub async fn send(&self, value: T) -> Result<(), SendError<T>> {
            self.0.send(value).await
      }

      pub async fn recv(&mut self) -> Option<U> {
            self.1.recv().await
      }
}