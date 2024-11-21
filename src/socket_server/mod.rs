use std::io;
//use std::io::ErrorKind;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use futures::SinkExt;
use futures::StreamExt;
use rocket::routes;
//use tokio::net::TcpListener;
//use tokio::net::TcpStream;
use tokio::sync::Mutex;
//use tokio_tungstenite::accept_async;
//use tokio_tungstenite::tungstenite::Message;
//use tokio_tungstenite::WebSocketStream;
use ws::stream::DuplexStream;
use crate::message::ServerMessage;
use crate::utils::ArcMut;
use crate::utils::new_arcmut;
use crate::utils::Tx;
use crate::utils::Rx;

/* 
pub struct SocketServerSenders {
      pub new_message: Tx<ServerMessage>,
      pub new_user: Tx<String>,
}

impl SocketServerSenders {

#[derive(Default)]
pub struct SendersBuilder {
      new_message: Option<Tx<ServerMessage>>,
      new_user: Option<Tx<String>>,
}

impl SendersBuilder {
      pub fn new_message(mut self, sender: Tx<ServerMessage>) -> SendersBuilder {
            self.new_message = Some(sender);
            self
      }

      pub fn new_user(mut self, sender: Tx<String>) -> SendersBuilder {
            self.new_user = Some(sender);
            self
      }

      pub fn build(self) -> SocketServerSenders {
            SocketServerSenders::new(
                  self.new_message.expect("No new message sender"),
                  self.new_user.expect("No new user sender")
            )
      }
}

      pub fn new(
            new_message: Tx<ServerMessage>,
            new_user: Tx<String>,
      ) -> Self
      {
            Self {
                  new_message,
                  new_user,
            }
      }

      pub fn builder() -> SendersBuilder {
            SendersBuilder::default()
      }
}

 */

/* 
pub struct ChatServerReceivers {
      pub new_message: Rx<ServerMessage>,
      pub new_user: Rx<String>,
}

impl ChatServerReceivers {

#[derive(Default)]
pub struct ReceiversBuilder {
      new_message: Option<Rx<ServerMessage>>,
      new_user: Option<Rx<String>>,
}

impl ReceiversBuilder {
      pub fn new_message(mut self, sender: Rx<ServerMessage>) -> ReceiversBuilder {
            self.new_message = Some(sender);
            self
      }

      pub fn new_user(mut self, sender: Rx<String>) -> ReceiversBuilder {
            self.new_user = Some(sender);
            self
      }

      pub fn build(self) -> ChatServerReceivers {
            ChatServerReceivers::new(
                  self.new_message.expect("No new message sender"),
                  self.new_user.expect("No new user sender")
            )
      }
}

      pub fn new(
            new_message: Rx<ServerMessage>,
            new_user: Rx<String>,
      ) -> Self
      {
            Self {
                  new_message,
                  new_user,
            }
      }

      pub fn builder() -> ReceiversBuilder {
            ReceiversBuilder::default()
      }
}

 */

 /* 
pub struct SocketServerReceivers {
      pub new_message: Rx<Message>,
}

 */
/* 
pub struct ChatServerSenders {
      pub new_message: Tx<Message>,
}

 */

/* 
pub struct SocketServerEvents {
      senders: ArcMut<SocketServerSenders>,
      receivers: SocketServerReceivers,
}

impl SocketServerEvents {
      pub fn new(senders: SocketServerSenders, receivers: SocketServerReceivers) -> Self {
            Self {
                  senders: new_arcmut(senders),
                  receivers: receivers,
            }
      }
}
 */

/* 
pub struct SocketServer {
      //clients: Arc<Mutex<HashMap<String, User>>>,
      id: String,
      state: ArcMut<State>,
      events: SocketServerEvents,
      addr: SocketAddr,
}


impl SocketServer {
      pub fn new(id: String, addr: SocketAddr, events: (SocketServerSenders,SocketServerReceivers)) -> Self {
            Self {
                  //clients: Arc::new(Mutex::new(HashMap::new())),
                  id,
                  state: new_arcmut(State::new()),
                  events: SocketServerEvents::new(events.0, events.1),
                  addr,
            }
      }

      pub async fn init(&mut self) {

            let listener = TcpListener::bind(self.addr.clone()).await.expect("Cannot bind");

            log::info!("Server listening on: {}", self.addr);

            loop {
                  let result = listener.accept().await;
                  if let Err(_) = result {continue}

                  let (tcp_stream, client_addr) = result.unwrap();

                  let state = self.state.clone();
                  //let events = self.events.clone();
                  let events = self.events.senders.clone();

                  tokio::spawn(async move {
                        log::info!("New connection request");

                        let result = Self::handle_connection(state, events, tcp_stream, client_addr).await;
                        if let Err(error) = result { log::error!("Connection Error: {}", error) };

                        log::info!("Connection ended");
                  });
            }
      }

      async fn handle_connection(state: ArcMut<State>, events: ArcMut<SocketServerSenders>, tcp_stream: TcpStream, client_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>>
      {      
            let ws_stream = match accept_async(tcp_stream).await {
                  Ok(ws_stream) => ws_stream,
                  Err(_) => return Err(
                        Box::new(std::io::Error::new(ErrorKind::NotConnected, "Error at websocket handshake"))
                  ),
            };
      
            let mut peer = match Peer::new(state.clone(), ws_stream, client_addr.clone()).await {
                  Ok(peer) => peer,
                  Err(_) => return Err(
                        Box::new(std::io::Error::new(ErrorKind::ConnectionAborted, "Error at creating channel"))
                  ),
            };
      
            log::info!("Connection request accepted");
      
            
            loop {
            tokio::select! {
                  Some(message) = peer.rx.recv() => peer.stream.send(Message::Text(message)).await?,
                  
                  fetch_result = peer.stream.next() => match fetch_result {
                        Some(Ok(Message::Close(_))) => break,
                        Some(Ok(Message::Text(msg))) => Self::process_user_message(events.clone(), &mut peer, msg).await,
                        Some(Err(_)) => log::error!("Error processing message"),
                        Some(Ok(_)) => (),
                        None => break,
                  }
            }
            }
      
            Ok(())
      
      }

      async fn process_user_message(events: ArcMut<SocketServerSenders>, peer: &mut Peer, msg: String) {
            
            log::info!("New message request");

            let result = serde_json::from_str::<ServerMessage>(&msg);
            if result.is_err() {
                  log::error!("Message not valid");
                  let _ = peer.stream.send(Message::text("Message not valid")).await;
                  return;
            };

            let message = result.unwrap();

            {
                  let events = events.lock().await;
                  let result = events.new_message.send(message);

                  if result.is_err() {
                        log::error!("Something gone wrong");
                        let _ = peer.stream.send(Message::text("Something gong wrong")).await;
                        return;
                  }
            }

            let _ = peer.stream.send(Message::text("Message sent succesfully")).await;

      }

      pub fn get_id(&self) -> String {
            self.id.clone()
      }

}


 */




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

/* struct Peer {
      stream: WebSocketStream<TcpStream>,
      rx: Rx<String>,
}

impl Peer {
      async fn new(state: Arc<Mutex<State>>, ws_stream: WebSocketStream<TcpStream>, client_addr: SocketAddr) -> io::Result<Peer>
      {     
            let (tx,rx) = tokio::sync::mpsc::unbounded_channel();
            let mut state = state.lock().await;
            state.peers.insert(client_addr, tx);
            
            Ok(
                  Peer {
                        stream: ws_stream,
                        rx,
                  }
            )
      }
} */

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