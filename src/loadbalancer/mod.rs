use crate::ChatServer;



pub struct LoadBalancer {
      chat_servers: Vec<ChatServer>
}

impl LoadBalancer {
      pub fn new() -> Self {
            Self {
                  chat_servers: vec![],
            }
      }

      pub fn add_server(&mut self, server: ChatServer) {
            self.chat_servers.push(server);
      }

      pub fn get_server(&mut self) -> Option<&mut ChatServer> {
            self.chat_servers.get_mut(0)
      }
}