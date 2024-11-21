use crate::Message;
use crate::User;

#[derive(Clone)]
pub struct ChatRoom 
{
    room_id: String,
    participants: Vec<User>,
    //messages: Vec<Message>,
}

impl ChatRoom 
{
    pub fn new(room_id: String) -> Self {
        Self {
            room_id,
            participants: vec![],
      //      messages: vec![],
        }
    }

    pub fn get_room_id(&self) -> String {
        self.room_id.clone()
    }

    pub fn add_participant(&mut self, user: User) {
        self.participants.push(user);
    }

    pub fn get_participants(&mut self) -> &Vec<User> {
        &self.participants
    }

    pub fn remove_participant(&mut self, user: User) {
        for (i, participant) in self.participants.iter().enumerate() {
            if participant.get_user_id() == user.get_user_id() {
                self.participants.remove(i);
                break;
            }
        }
    }

    pub fn send_message(&mut self, message: Message) {
//        self.messages.push(message);
        println!("\n{} {}\t> {}\n", message.get_room_id(), message.get_sender_id(), message.get_content());
    }
}
