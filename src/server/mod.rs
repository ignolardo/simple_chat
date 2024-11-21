use std::{collections::HashMap, error::Error, io::{self, ErrorKind}, net::SocketAddr, sync::Arc};
use tokio::{net::{TcpListener, TcpStream}, sync::{mpsc, Mutex}};
use tokio_tungstenite::{accept_async, tungstenite::{accept, protocol::Message}, WebSocketStream};
use futures::{stream::SplitSink, SinkExt, StreamExt};
use serde::{Serialize, Deserialize};
use tokio_util::codec::{Framed, LinesCodec};
use crate::User;

pub type IOResult<T> = std::io::Result<T>;

type Clients = Arc<Mutex<HashMap<String,User>>>;
type Sender = SplitSink<WebSocketStream<TcpStream>, Message>;
type _Receiver = SplitSink<WebSocketStream<TcpStream>, Message>;

type Tx = tokio::sync::mpsc::UnboundedSender<String>;
type Rx = tokio::sync::mpsc::UnboundedReceiver<String>;

type State = Arc<Mutex<Shared>>;


enum RequestResult {
      Ok,
      Close,
      Error,
}

#[derive(Serialize, Deserialize, Debug)]
struct MessageRequest {
      message: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RegisterRequest {
      user: String,
}

struct Shared {
      peers: HashMap<SocketAddr, Tx>
}

impl Shared {
      fn new() -> Self {
            Self {
                  peers: HashMap::new()
            }
      }

      async fn broadcast(&mut self, addr: SocketAddr, message: &str) {
            for peer in self.peers.iter_mut() {
                  if *peer.0 != addr {
                        let _ = peer.1.send(message.into());
                  }
            }
      }
}

struct Peer {
      lines: Framed<TcpStream, LinesCodec>,
      rx: Rx,
}

impl Peer {
      //, lines: Framed<TcpStream, LinesCodec>
      async fn new(state: State, ws: WebSocketStream<TcpStream>, addr: SocketAddr) -> io::Result<Peer> {
            //let addr = lines.get_ref().peer_addr()?;
            
            let (tx, rx) = mpsc::unbounded_channel();
            
            state.lock().await.peers.insert(addr, tx);

            Ok(Peer {lines,rx} )
      }
}

pub struct SocketServer {
      clients: Clients,
      addr: SocketAddr,
}

impl SocketServer {
      pub fn new(addr: SocketAddr) -> Self {
            Self {
                  clients: Arc::new(Mutex::new(HashMap::new())),
                  addr
            }
      }

      pub async fn init(&self) {
            let listener = TcpListener::bind(&self.addr).await.expect("Cannot bind");
            log::info!("Server listening on: {}", self.addr);
            println!("Server listening on: {}", self.addr);

            /* let register_routes = warp::path("")
                  .and(warp::ws())
                  .and(warp::path::param())
                  .and_then(move |ws,a: String| {
                        ws.on_upgrade(move |socket| {
                              let (ws_sender, mut ws_receiver) = socket.split();
                              let (sender, receiver) = mpsc::unbounded();
                              let receiver = tokio
                        })
                  });
             */        

            let state = Arc::new(Mutex::new(Shared::new()));

            while let Ok((stream, addr)) = listener.accept().await {
                  let state = Arc::clone(&state);
                  tokio::spawn(async move {
                        if let Err(_) = Self::handle_connection(stream, state, addr).await {
                              println!("Error when connecting");
                        }
                  });
            }            
      }

      async fn handle_connection(stream: TcpStream, state: State, addr: SocketAddr) -> Result<(), Box<dyn Error>>{
            
            let ws_stream = match accept_async(stream).await {
                  Ok(ws_stream) => ws_stream,
                  Err(e) => {
                        log::error!("Error at websocket handshake: {}", e);
                        return Err(Box::new(std::io::Error::from(ErrorKind::NotConnected)));
                  },
            };
            
            //let (mut sender, mut receiver) = ws_stream.split(); 

            let mut lines = Framed::new(stream, LinesCodec::new());

            let mut peer = Peer::new(state.clone(), ws_stream, addr).await?;

            {
                  let mut state = state.lock().await;
                  state.broadcast(addr, "New user in the server").await;
            }

            loop {}

            /* while let Some(req) = receiver.next().await {
                  match Self::handle_request(req, &mut sender).await {
                        RequestResult::Ok | RequestResult::Error => (),
                        RequestResult::Close => break,
                  }
            }; */

            Ok(())
      }

      /* async fn handle_request(message: Result<Message, Error>, sender: &mut Sender) -> RequestResult {
            match message {
                  Ok(Message::Text(msg)) => {
                        if let Ok(msg_request) = serde_json::from_str::<MessageRequest>(msg.as_str()) {
                              if let Err(_) = sender.send(Message::Text(format!("Message request : {}", msg_request.message))).await {
                                    log::error!("Error sending back message");
                              };
                              return RequestResult::Ok;
                        };

                        if let Ok(reg_request) = serde_json::from_str::<RegisterRequest>(msg.as_str()) {
                              if let Err(_) = sender.send(Message::Text(format!("Register request : {}", reg_request.user))).await {
                                    log::error!("Error sending back message");
                              };
                              return RequestResult::Ok;
                        };
                        
                        if let Err(_) = sender.send(Message::Text(format!("Invalid request..."))).await {
                              log::error!("Error sending back message");
                        };

                        return RequestResult::Error;
                  },
                  Ok(Message::Close(_)) => RequestResult::Close,
                  Ok(_) => RequestResult::Error,
                  Err(e) => {
                        log::error!("Error at message processing: {}", e);
                        return RequestResult::Error;
                  }
            }
      } */
}

/* #[tokio::main]
pub async fn main()
{
      env_logger::init();

      let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

      let port = 3000;
      let addr = format!("127.0.0.1:{}",port).to_string();
      let addr: SocketAddr = addr.parse().expect("Invalid Address");

      let listener = TcpListener::bind(&addr).await.expect("Cannot bind");
      
      log::info!("Server listening on: {}", addr);
      println!("Server listening on: {}", addr);

      while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(handle_connection(stream));
      }
} */


/* async fn handle_connection(stream: TcpStream)
{
      let ws_stream = match accept_async(stream).await {
            Ok(ws_stream) => ws_stream,
            Err(e) => {
                  log::error!("Error at websocket handshake: {}", e);
                  return;
            },
      };

      let (mut sender, mut receiver) = ws_stream.split();

      while let Some(message) = receiver.next().await {
            match message {
                  Ok(Message::Text(text)) => {
                        if let Err(_) = sender.send(Message::Text(format!("Received text: {}", text))).await {
                              log::error!("Error sending back message");
                        }
                  },
                  Ok(Message::Close(_)) => break,
                  Ok(_) => (),
                  Err(e) => {
                        log::error!("Error at message processing: {}", e);
                        break;
                  }
            }
      };
} */