use crate::message::ServerMessage;
use crate::servers::MessageServer;
use crate::utils::new_arcmut;
use crate::ChatRoom;
use crate::Message;
use crate::User;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use crate::utils::Tx;
use crate::utils::Rx;


pub struct ChatRoomsDatabase {
      chat_rooms: HashMap<String,ChatRoom>,
}

impl ChatRoomsDatabase {
      pub fn new() -> Self {
            Self {
                  chat_rooms: HashMap::new(),
            }
      }

      pub fn find_room_by_id(&mut self, id: String) -> Option<&mut ChatRoom> {
            self.chat_rooms.get_mut(&id)
      }

      pub fn add_chat_room(&mut self, room: ChatRoom) {
            self.chat_rooms.insert(room.get_room_id().clone(), room);
      }
}



pub struct MessagesDatabase {
      messages: HashMap<String, Vec<Message>>,
}

impl MessagesDatabase {
      pub fn new() -> Self {
            Self {
                  messages: HashMap::new(),
            }
      }

      pub fn store_message(&mut self, message: Message) {
            if let Some(room_messages) = self.messages.get_mut(&message.get_room_id()) {
                  //println!("New Message:\n\troom: {}\tuser: {}\tcontent: {}", message.get_room_id(), message.get_sender_id(), message.get_content());
                  room_messages.push(message);
            } else {
                  //println!("New Message:\n\troom: {}\tuser: {}\tcontent: {}", message.get_room_id(), message.get_sender_id(), message.get_content());
                  self.messages.insert(message.get_room_id(), vec![message]);
            }
      }

      pub fn get_room_messages(&self, room_id: String) -> Option<&Vec<Message>> {
            self.messages.get(&room_id)
      }
}


pub struct ChatServer
{
      id: String,
      chat_rooms_db: ChatRoomsDatabase,
      messages_db: MessagesDatabase,
      message_server: Arc<Mutex<MessageServer>>,
      message_server_channel: (Tx<crate::Message>, Rx<ServerMessage>),
      loadbalancer_channel: (Tx<Message>, Rx<Message>),
}

impl ChatServer {

      pub fn new(id: String, chat_rooms_db: ChatRoomsDatabase, messages_db: MessagesDatabase, loadbalancer_channel: (Tx<Message>, Rx<Message>)) -> Self {
            let (
                  _ms_cs_tx,
                  _cs_ms_rx,
                  cs_ms_tx,
                  ms_cs_rx,
            ) = create_message_channels();
            let message_server = MessageServer::new(String::from("1234"), SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 8000));

            Self {
                  id,
                  chat_rooms_db,
                  messages_db,
                  message_server: new_arcmut(message_server),
                  message_server_channel: (cs_ms_tx, ms_cs_rx),
                  loadbalancer_channel,
            }
      }

      pub async fn init(&mut self) {
            let message_server = self.message_server.clone();
            tokio::spawn(async move {
                  let mut message_server = message_server.lock().await;
                  let _ = message_server.init().await;
            });
            
            loop {
                  tokio::select! {
                        Some(msg) = self.message_server_channel.1.recv() => self.process_server_message(msg.clone())
                  }
            }
      }

      pub fn get_server_id(&self) -> String {
            self.id.clone()
      }

      pub fn send_message(&mut self, message: Message) -> Result<(),()> {

            let room_id = message.get_room_id();

            if let Some(room) = self.chat_rooms_db.find_room_by_id(room_id) {
                  let _ = self.loadbalancer_channel.0.send(message.clone());
                  room.send_message(message.clone());
                  self.store_message_in_history(message.clone());
                  Ok(())
            } else {
                  Err(())
            }

      }

      fn process_server_message(&mut self, msg: ServerMessage) {
            let result = self.send_message(msg.into());
            if let Err(_) = result {
                  log::error!("Incorrect chat room");
                  return;
            }
            log::info!("Message delivered and stored in database");
      }

      pub fn add_user_to_room(&mut self, user: User, room_id: String) -> Result<(),()> {
            if let Some(room) = self.chat_rooms_db.find_room_by_id(room_id) {
                  room.add_participant(user);
                  Ok(())
            } else {
                  Err(())
            }
      }

      pub fn get_room_messages(&mut self, room_id: String) -> Option<&Vec<Message>> {
            self.messages_db.get_room_messages(room_id)
      }

      pub fn get_room_participants(&mut self, room_id: String) -> Option<&Vec<User>> {
            if let Some(room) = self.chat_rooms_db.find_room_by_id(room_id) {
                  Some(room.get_participants())
            } else {
                  None
            }
      }
      
      fn store_message_in_history(&mut self, message: Message) {
            self.messages_db.store_message(message);
      }
}


fn create_message_channels() -> (
      Tx<ServerMessage>,
      Rx<crate::Message>,
      Tx<crate::Message>,
      Rx<ServerMessage>,
)
{
      // Socket server Messages to chat server
      let ms_to_cs = mpsc::unbounded_channel();
      let cs_to_ms = mpsc::unbounded_channel();

      (
            ms_to_cs.0,
            cs_to_ms.1,
            cs_to_ms.0,
            ms_to_cs.1,
      )
}