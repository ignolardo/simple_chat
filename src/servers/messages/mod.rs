use std::net::SocketAddr;
use std::collections::HashMap;
use std::ops::Deref;
use futures::SinkExt;
use futures::StreamExt;
use rocket::routes;
use ws::stream::DuplexStream;
use crate::message::ServerMessage;
use crate::utils;
use crate::Message;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc;

/* 
struct State {
      //peers: HashMap<SocketAddr, Tx<String>>,
      peers: HashMap<SocketAddr, Sender<Message>>,
}

impl State {
      fn new() -> Self {
            Self {
                  peers: HashMap::new(),
            }
      }

      fn add_peer(&mut self, addr: SocketAddr, mut stream: Stream<Message, ServerMessage>) {
            let _tx = stream.0.clone();
            tokio::spawn(async move {
                  loop{
                        tokio::select! {
                              result = stream.recv() => match result {
                                    Some(msg) => println!("{}", msg.get_content()),
                                    None => break,
                              }
                        }
                  }
            });
            /* self.peers.insert(addr.clone(), new_arcmut(stream));
            let stream = self.peers.get(&addr).expect("nothing...").clone();
            tokio::spawn(async move {
                  let mut stream = stream.lock().await;
                  loop {
                        tokio::select! {
                              result = stream.recv() => match result {
                                    Some(_msg) => println!("New message"),
                                    None => break,
                              }
                        }
                  }
            }); */
      }

      fn remove_peer(&mut self, addr: SocketAddr) {
            self.peers.remove(&addr);

      }

      pub async fn send(&self, addr: SocketAddr, message: Message) -> Result<(),()> {
            let result = self.peers.get(&addr);
            if result.is_none() {return Err(())}
            let tx = result.unwrap();
            let result = tx.send(message).await;
            if result.is_err() {return Err(())}
            Ok(())
      }

      /* fn broadcast(&mut self, sender_addr: SocketAddr, message: String) {
            for (addr,tx) in self.peers.iter() {
                  if *addr != sender_addr {
                        tx.send(message.clone()).unwrap();
                  }
            }
      } */
}

struct Peer {
      stream: DuplexStream,
      //rx: Rx<String>,
      server_stream: utils::channels::Stream<ServerMessage,Message>
}

impl Peer {
      async fn new(state: ArcMut<State>, stream: DuplexStream, client_addr: SocketAddr) -> io::Result<Self>
      {     
            //let (tx,rx) = tokio::sync::mpsc::unbounded_channel();
            let (stream1, stream2) = utils::channels::double_channel::<ServerMessage,Message>(10);
            
            {
                  let mut state = state.lock().await;
                  state.add_peer(client_addr, stream2);
            }


            log::info!("Peer created");
            Ok(
                  Self {
                        stream,
                        server_stream: stream1,
                  }
            )
      }
}

impl Drop for Peer {
      fn drop(&mut self) {
          log::info!("Peer deleted");
      }
}

 */
enum EventType {
      Message(ServerMessage),
      Connection(SocketAddr, Sender<Message>),
}


pub struct MessageServer {
      id: String,
      //state: ArcMut<State>,
      connections: HashMap<SocketAddr,Sender<Message>>,
      //message_channel: Arc<(Tx<ServerMessage>, Rx<crate::Message>)>,
}


impl MessageServer 
{
      pub fn new(id: String) -> Self {
            Self {
                  id,
                  //state: new_arcmut(State::new()),
                  connections: HashMap::new(),
                  //message_channel: Arc::new(message_channel),
            }
      }

      pub async fn init(&mut self) -> Result<(), rocket::Error> {

            let (tx,mut rx) = mpsc::channel::<EventType>(1);

            //let state = self.state.clone();
            let thread = tokio::spawn(async move {
                  let _ = rocket::build()
                        .mount("/", routes![message_channel])
                        //.manage(state)
                        .manage(tx)
                        .launch()
                        .await;
            });

            loop {
                  tokio::select! {
                        result = rx.recv() => match result {
                              Some(EventType::Message(msg)) => println!("{}", msg.get_content()),
                              Some(EventType::Connection(addr,tx)) => {self.connections.insert(addr, tx);},
                              None => break,
                        }
                  }
            }

            let _ = thread.await;

            Ok(())
      }
      
      pub fn get_id(&self) -> String {
            self.id.clone()
      }

      /* pub async fn send(&self, user_id: String, message: Message) {
            let state = self.state.lock().await;

      } */
}



#[get("/")]
fn message_channel(
      remote_addr: SocketAddr, 
      //state: &rocket::State<ArcMut<State>>,
      sender: &rocket::State<Sender<EventType>>,
      ws: ws::WebSocket
) 
-> ws::Channel<'static> 
{     

      
      //let state = state.deref().clone();
      let sender = sender.deref().clone();
      ws.channel(move |stream| Box::pin(handle_connection(sender, remote_addr, stream)))
}

async fn handle_connection(
      //state: ArcMut<State>,
      sender: Sender<EventType>,
      remote_addr: SocketAddr, 
      mut stream: DuplexStream,
)
-> Result<(), ws::result::Error>
{

      log::info!("Connection Opened");

      /* let mut peer = match Peer::new(state.clone(), stream, remote_addr.clone()).await {
            Ok(peer) => peer,
            Err(_) => return Err(
                  ws::result::Error::ConnectionClosed
            ),
      }; */

      
      let (mut peer_stream, server_stream) = utils::channels::double_channel::<ServerMessage,Message>(10);
      
      /* {
            let mut state = state.lock().await;
            state.add_peer(remote_addr.clone(), state_stream);
      } */

      let _ = sender.send(EventType::Connection(remote_addr.clone(), server_stream.0));
      
      loop {
            rocket::tokio::select! {

                  Some(msg) = peer_stream.recv() => stream.send(ws::Message::text(msg)).await?,

                  result = &mut stream.next() => match result {
                        Some(Ok(ws::Message::Close(_))) => break,
                        Some(Ok(ws::Message::Text(msg))) => { let _ = process_user_message(msg, &mut stream, sender.clone()).await; },
                        Some(Err(_)) => { let _ = stream.send(ws::Message::text("Error")).await; },
                        None => return Err(ws::result::Error::ConnectionClosed),
                        _ => (),
                  }
            }
      }

      /* {
            let mut state = state.lock().await;
            state.remove_peer(remote_addr.clone());
      } */

      log::info!("Connection Closed");
      
      Ok(())
}


async fn process_user_message(msg: String, stream: &mut DuplexStream, sender: Sender<EventType>) -> Result<(),()> {
      
      log::info!("New message request");

      let result = serde_json::from_str::<ServerMessage>(&msg);
      if result.is_err() {
            log::error!("Message not valid");
            let _ = stream.send(ws::Message::text("Message not valid")).await;
            return  Err(());
      };

      let message = result.unwrap();

      //let result = message_sender.send(message);
      let result = sender.send(EventType::Message(message)).await;

      if result.is_err() {
            log::error!("Something gone wrong");
            let _ = stream.send(ws::Message::text("Something gone wrong")).await;
            return Err(());
      }

      let _ = stream.send(ws::Message::text("Message received")).await;

      Ok(())
}