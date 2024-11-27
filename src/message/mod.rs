use serde::Serialize;
use serde::Deserialize;


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Message 
{
    message_id: String,
    sender_id: String,
    room_id: String,
    content: String,
    time_stamp: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ServerMessage {
    sender_id: String,
    room_id: String,
    content: String,
}


impl Message 
{
    pub fn new(message_id: String, sender_id: String, room_id: String, content: String) -> Result<Self, ()> {
        let start = std::time::SystemTime::now();
        let since_epoch = start.duration_since(std::time::UNIX_EPOCH);

        if let Err(_) = since_epoch {
            return Err(());
        };

        let since_epoch = since_epoch.unwrap();

        Ok(Self {
            message_id,
            sender_id,
            room_id,
            content,
            time_stamp: since_epoch.as_secs(),
        })
    }

    pub fn get_message_id(&self) -> String {
        self.message_id.clone()
    }

    pub fn get_sender_id(&self) -> String {
        self.sender_id.clone()
    }

    pub fn get_room_id(&self) -> String {
        self.room_id.clone()
    }

    pub fn get_content(&self) -> String {
        self.content.clone()
    }

    pub fn get_time_stamp(&self) -> u64 {
        self.time_stamp.clone()
    }
}


impl Into<String> for Message {
    fn into(self) -> String {
        let result = serde_json::to_string(&self);
        match result {
            Ok(message) => message,
            Err(_) => String::from(""),
        }
    }
}


impl ServerMessage {
    pub fn get_sender_id(&self) -> String {
        self.sender_id.clone()
    }

    pub fn get_room_id(&self) -> String {
        self.room_id.clone()
    }

    pub fn get_content(&self) -> String {
        self.content.clone()
    }
}

impl Into<Message> for ServerMessage {
    fn into(self) -> Message {
        Message::new(String::from("1234"), self.sender_id, self.room_id, self.content).unwrap()
    }
}