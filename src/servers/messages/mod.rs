use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::collections::HashMap;
use futures::SinkExt;
use futures::StreamExt;
use reqwest::StatusCode;
use rocket::routes;
use tokio::sync::Mutex;
use ws::stream::DuplexStream;
use crate::message::ServerMessage;
use crate::utils::channels::DoubleChannelPool;
use crate::utils::channels::Stream;
use crate::Message;
use tokio::sync::mpsc::Sender;


static USER_MAPPING_SERVER_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 3000);

pub struct MessageServer {
      id: String,
      addr: SocketAddr,
      connections: HashMap<String,Sender<Message>>,
}

impl MessageServer 
{
      pub fn new(id: String, addr: SocketAddr) -> Self {
            Self {
                  id,
                  addr,
                  connections: HashMap::new(),
            }
      }

      pub async fn init(&mut self) -> Result<(), rocket::Error> {

            let addr = self.addr.clone();

            let (channel_pool,mut msg_receiver,mut conn_receiver) = DoubleChannelPool::<Message,ServerMessage,String>::new();

            let thread = tokio::spawn(async move {
                  let _ = rocket::build()
                        .mount("/", routes![message_channel])
                        .manage(Mutex::new(channel_pool))
                        .manage(addr)
                        .launch().await;
            });

            loop {
                  tokio::select! {
                        result = msg_receiver.recv() => match result {
                              Some(msg) => println!("{} >\t{}", msg.get_sender_id(), msg.get_content()),
                              None => break,
                        },
                        result = conn_receiver.recv() => match result {
                              Some((id,sender)) => {self.register_user(id, sender).await;},
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

      async fn register_user(&mut self, user_id: String, sender: Sender<Message>) {
                        
            // Register connection in local hashmap
            self.connections.insert(user_id.clone(), sender);

            // Register connection in user mapping service
            let client = reqwest::Client::new();
            let result = client.put(format!("http://{}/{}/{}", USER_MAPPING_SERVER_ADDR, user_id, self.addr))
                  .send()
                  .await;
            if result.is_err() {
                  log::info!("Registering user failed");
                  return;
            }
            let status = result.unwrap().status();
            if status != StatusCode::OK {
                  log::info!("Registering user failed");
                  return;
            }
            log::info!("User registered succesfully");
      }

      pub async fn send_to_local_user(&self, user_id: String, message: Message) {
            let result = self.connections.get(&user_id);
            if result.is_none() {return;}
            let sender = result.unwrap();
            let _ = sender.send(message).await;
      }
}


#[get("/<id>")]
async fn message_channel(
      remote_addr: SocketAddr,
      id: String,
      //state: &rocket::State<ArcMut<State>>,
      //sender: &rocket::State<Sender<EventType>>,
      channel_pool: &rocket::State<Mutex<DoubleChannelPool<Message,ServerMessage,String>>>,
      ws: ws::WebSocket
) 
-> ws::Channel<'static> 
{     
      let channel_stream = channel_pool
            .lock()
            .await
            .join(id.clone())
            .await;

      ws.channel(move |stream| Box::pin(handle_connection(channel_stream, remote_addr, id.clone(), stream)))
}

async fn handle_connection(
      mut channel_stream: Stream<ServerMessage,Message>,
      _remote_addr: SocketAddr,
      user_id: String,
      mut stream: DuplexStream,
)
-> Result<(), ws::result::Error>
{
      
      log::info!("Connection Opened");
      log::info!("User id: {}", user_id);
      
      loop {
            rocket::tokio::select! {

                  Some(msg) = channel_stream.recv() => stream.send(ws::Message::text(msg)).await?,

                  result = &mut stream.next() => match result {
                        Some(Ok(ws::Message::Close(_))) => break,
                        Some(Ok(ws::Message::Text(msg))) => { let _ = process_user_message(msg, &mut stream, channel_stream.0.clone()).await; },
                        Some(Err(_)) => { let _ = stream.send(ws::Message::text("Error")).await; },
                        None => return Err(ws::result::Error::ConnectionClosed),
                        _ => (),
                  }
            }
      }

      log::info!("Connection Closed");
      
      Ok(())
}


async fn process_user_message(msg: String, stream: &mut DuplexStream, sender: Sender<ServerMessage>) -> Result<(),()> {
      
      log::info!("New message request");
      
      let result = serde_json::from_str::<ServerMessage>(&msg);
      if result.is_err() {
            log::error!("Message not valid");
            let _ = stream.send(ws::Message::text("Message not valid")).await;
            return  Err(());
      };

      let message = result.unwrap();

      let result = sender.send(message).await;

      if result.is_err() {
            log::error!("Something gone wrong");
            let _ = stream.send(ws::Message::text("Something gone wrong")).await;
            return Err(());
      }

      let _ = stream.send(ws::Message::text("Message received")).await;

      Ok(())
}

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