use std::collections::HashMap;
use std::sync::Arc;
use futures::SinkExt;
use futures::StreamExt;

use crate::utils::new_arcmut;
use crate::utils::ArcMut;
use crate::ChatServer;
use crate::Message;
use crate::utils::Tx;
use crate::utils::Rx;

type MessageChannel = (Tx<Message>, Rx<Message>);

pub struct LoadBalancer<'a> {
      chat_servers: HashMap<String, &'a ChatServer>,
      chat_server_channels: ArcMut<HashMap<String, MessageChannel>>,
      users_table: Arc<HashMap<String, String>>,
}

impl<'a> LoadBalancer<'a> {
      pub fn new() -> Self {
            Self {
                  chat_servers: HashMap::new(),
                  chat_server_channels: new_arcmut(HashMap::new()),
                  users_table: Arc::new(HashMap::new()),
            }
      }

      pub async fn init(&mut self) {

            let _ = rocket::build()
                  .mount("/", routes![message_channel])
                  //.manage(state)
                  .launch()
                  .await;
      }

      fn send_to_server(server_id: String, message: Message) -> Result<(),()> {
            Ok(())
      }

      pub async fn add_server(&mut self, server: &'a ChatServer, channel: (Tx<Message>, Rx<Message>)) {
            self.chat_server_channels.lock().await.insert(server.get_server_id(), channel);
            self.chat_servers.insert(server.get_server_id(), server);
      }

      pub fn get_server(&mut self) -> Option<&mut &'a ChatServer> {
            self.chat_servers.values_mut().next()
      }
}


#[get("/")]
async fn message_channel(
      ws: ws::WebSocket
) -> ws::Channel<'static>
{
      ws.channel(move |mut stream| Box::pin(async move {
            loop {
                  tokio::select! {
                        result = stream.next() => match result {
                              Some(Ok(ws::Message::Close(_))) => break,
                              Some(Ok(ws::Message::Text(msg))) => {let _ = stream.send(ws::Message::text(msg));},
                              Some(Err(_)) => continue,
                              None => break,
                              _ => (),
                        }
                  }
            }
            Ok(())
      }))
}