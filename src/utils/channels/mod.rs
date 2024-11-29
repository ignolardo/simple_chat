use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::Receiver;


pub struct Stream<T,U>(pub mpsc::Sender<T>, pub mpsc::Receiver<U>);

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


pub enum EventType<T,U,C> {
      Message(T),
      Connection(C, Sender<U>),
}


/*
      Bidirectional channel pool.

      The main node is the one who creates the pool.
      When clients join the pool, the main node receives the connections.

      T: Type to send to clients
      U: Type to receive from clients
      C: Extra data from new connections
*/
pub struct DoubleChannelPool<T,U,C>
{
      sender: Sender<EventType<U,T,C>>,
      size: usize,
}

impl<T,U,C> DoubleChannelPool<T,U,C>
where T: Send + 'static, U: Send + 'static, C: Send + 'static
{

      pub fn new(size: usize) -> (Self,Receiver<EventType<U,T,C>>) {
            let (msg_sender,msg_receiver) = tokio::sync::mpsc::channel::<EventType<U,T,C>>(size);

            let channel_pool = Self {
                  sender: msg_sender,
                  size
            };
            (channel_pool,msg_receiver)
      }

      pub async fn join(&mut self, data: C) -> DoubleChannelPoolStream<U,T,C> {
            //let (sender,receiver) = tokio::sync::mpsc::channel::<T>(self.size);
            let channel_stream = DoubleChannelPoolStream::<U,T,C>::new(self.size, self.sender.clone(), data).await;
            //let _ = self.conn_sender.send((data,sender)).await;
            channel_stream
      }

}

/*
      Like a normal Stream, but it sends EventTypes.
      send method takes a value of the send type and not of EventType.
*/
pub struct DoubleChannelPoolStream<T,U,C>(Sender<EventType<T,U,C>>, Receiver<U>);

impl<T,U,C> DoubleChannelPoolStream<T,U,C>
where T: Send + 'static, U: Send + 'static, C: Send + 'static
{
      pub async fn new(size: usize, sender: Sender<EventType<T,U,C>>, data: C) -> Self {
            let (peer_s,peer_r) = tokio::sync::mpsc::channel::<U>(size);
            let _ = sender.send(EventType::Connection(data, peer_s)).await;
            Self(sender,peer_r)
      }

      pub async fn send(&self, value: T) -> Result<(), SendError<EventType<T, U, C>>> {
            self.0.send(EventType::Message(value)).await
      }

      pub async fn recv(&mut self) -> Option<U> {
            self.1.recv().await
      }
}