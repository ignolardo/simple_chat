use std::io;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use futures::SinkExt;
use futures::StreamExt;
use rocket::routes;
use tokio::sync::Mutex;
use ws::stream::DuplexStream;
use crate::message::ServerMessage;
use crate::utils::ArcMut;
use crate::utils::new_arcmut;
use crate::utils::Tx;
use crate::utils::Rx;

struct State {
      peers: HashMap<SocketAddr, Tx<String>>,
}

impl State {
      fn new() -> Self {
            Self {
                  peers: HashMap::new(),
            }
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
      rx: Rx<String>,
}

impl Peer {
      async fn new(state: Arc<Mutex<State>>, stream: DuplexStream, client_addr: SocketAddr) -> io::Result<Self>
      {     
            let (tx,rx) = tokio::sync::mpsc::unbounded_channel();
            
            {
                  let mut state = state.lock().await;
                  state.peers.insert(client_addr, tx);
            }
            log::info!("Peer created");
            Ok(
                  Self {
                        stream,
                        rx,
                  }
            )
      }
}


pub struct MessageServer {
      id: String,
      state: ArcMut<State>,
      message_channel: Arc<(Tx<ServerMessage>, Rx<crate::Message>)>,
}


impl MessageServer 
{
      pub fn new(id: String, message_channel: (Tx<ServerMessage>, Rx<crate::Message>)) -> Self {
            Self {
                  id,
                  state: new_arcmut(State::new()),
                  message_channel: Arc::new(message_channel),
            }
      }

      pub async fn init(&mut self) -> Result<(), rocket::Error> {
            let _ = rocket::build()
                  .mount("/", routes![message_channel])
                  .manage(self.state.clone())
                  .manage(self.message_channel.clone())
                  .launch()
                  .await?;

            Ok(())
      }
      
      pub fn get_id(&self) -> String {
            self.id.clone()
      }
}



#[get("/")]
fn message_channel(
      remote_addr: SocketAddr, 
      state: &rocket::State<ArcMut<State>>, 
      message_channel: &rocket::State<Arc<(Tx<ServerMessage>, Rx<crate::Message>)>>, 
      ws: ws::WebSocket
) 
-> ws::Channel<'static> 
{     

      
      let state = state.deref().clone();
      let sender = message_channel.0.clone();
      ws.channel(move |stream| Box::pin(handle_connection(state, remote_addr, stream, sender)))
}

async fn handle_connection(
      state: ArcMut<State>, 
      remote_addr: SocketAddr, 
      stream: DuplexStream, 
      message_sender: Tx<ServerMessage>
)
-> Result<(), ws::result::Error>
{

      let mut peer = match Peer::new(state.clone(), stream, remote_addr.clone()).await {
            Ok(peer) => peer,
            Err(_) => return Err(
                  ws::result::Error::ConnectionClosed
            ),
      };

      loop {
            rocket::tokio::select! {

                  Some(msg) = peer.rx.recv() => peer.stream.send(ws::Message::text(msg)).await?,

                  result = peer.stream.next() => match result {
                        Some(Ok(ws::Message::Close(_))) => break,
                        Some(Ok(ws::Message::Text(msg))) => { let _ = process_user_message(msg, &mut peer, &message_sender).await; },
                        Some(Err(_)) => { let _ = peer.stream.send(ws::Message::text("Error")).await; },
                        None => return Err(ws::result::Error::ConnectionClosed),
                        _ => (),
                  }
            }
      }
      
      Ok(())
}


async fn process_user_message(msg: String, peer: &mut Peer, message_sender: &Tx<ServerMessage>) -> Result<(),()> {
      


      log::info!("New message request");

      let result = serde_json::from_str::<ServerMessage>(&msg);
      if result.is_err() {
            log::error!("Message not valid");
            let _ = peer.stream.send(ws::Message::text("Message not valid")).await;
            return  Err(());
      };

      let message = result.unwrap();

      {
            let result = message_sender.send(message);

            if result.is_err() {
                  log::error!("Something gone wrong");
                  let _ = peer.stream.send(ws::Message::text("Something gong wrong")).await;
                  return Err(());
            }
      }

      let _ = peer.stream.send(ws::Message::text("Message received")).await;

      Ok(())
}