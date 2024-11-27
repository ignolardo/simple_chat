use tokio::sync::mpsc::{self, error::SendError};
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::Receiver;

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

/* pub enum EventType<T,U> {
      Message(U),
      Connection(String, Sender<T>),
} */

pub struct DoubleChannelPool<T,U,C>{
      sender: Sender<U>,
      conn_sender: Sender<(C,Sender<T>)>,
}

impl<T,U,C> DoubleChannelPool<T,U,C> {

      pub fn new() -> (Self,Receiver<U>,Receiver<(C,Sender<T>)>) {
            let (msg_sender,msg_receiver) = tokio::sync::mpsc::channel::<U>(10);
            let (conn_sender,conn_receiver) = tokio::sync::mpsc::channel::<(C,Sender<T>)>(10);

            let channel_pool = Self {
                  sender: msg_sender,
                  conn_sender,
            };
            (channel_pool,msg_receiver,conn_receiver)
      }

      pub async fn join(&mut self, data: C) -> Stream<U,T> {
            let (sender,receiver) = tokio::sync::mpsc::channel::<T>(10);
            let _ = self.conn_sender.send((data,sender)).await;
            Stream(self.sender.clone(),receiver)
      }


}