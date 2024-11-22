use std::collections::HashMap;
use std::sync::Arc;
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
     /*        let receivers_iter = self.chat_server_channels
                  .clone()
                  .lock().await
                  .iter_mut()
                  .map(|(_,(_,r))| r);
            
            for receiver in receivers_iter {
                  let channels = self.chat_server_channels.clone();
                  let users_table = self.users_table.clone();
                  tokio::spawn(async move {
                        loop {
                              tokio::select! {
                                    Some(msg) = receiver.recv() => {
                                          let server_id = users_table.get(&msg.get_sender_id());
                                          if let None = server_id {
                                                continue;
                                          };
                                          let server_id = server_id.unwrap();
                                          
                                          let channels = channels.lock().await;
                                          let channel = channels.get(server_id);
                                          if let None = channel {
                                                continue;
                                          }
                                          let channel = channel.unwrap();
                                          let _ = channel.0.send(msg);
                                    }
                              }
                        }
                  });
            } */

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