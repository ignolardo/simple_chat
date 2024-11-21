//use simple_chat::SocketServer;
//use std::net::SocketAddr;
use simple_chat::ChatRoom;
//use simple_chat::socket_server::SocketServer;
use simple_chat::ChatServer;
use simple_chat::chatserver::ChatRoomsDatabase;
use simple_chat::chatserver::MessagesDatabase;

//static PORT: i32 = 3000;

#[tokio::main]
async fn main()
{
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let chat_room1 = ChatRoom::new(String::from("1234"));
    let mut rooms_db = ChatRoomsDatabase::new();
    rooms_db.add_chat_room(chat_room1);
    
    let messages_db = MessagesDatabase::new();

    let mut chat_server = ChatServer::new(String::from("1234"), rooms_db, messages_db);

    chat_server.init().await;
}


/* fn main() -> Result<(),()>
{
    let chatroom1 = ChatRoom::new(String::from("1234"));
    let chatroom_id = chatroom1.get_room_id();

    let user1 = User::new(
        String::from("7890"),
        String::from("John"),
        String::from("password"),
    );
    
    let user2 = User::new(
        String::from("4389"),
        String::from("Maria"),
        String::from("password"),
    );
    


    let mut rooms_db = ChatRoomsDatabase::new();
    let messages_db = MessagesHistoryDatabase::new();

    rooms_db.add_chat_room(chatroom1.clone());

    let mut load_balancer = LoadBalancer::new();
    let server1 = ChatServer::new(String::from("1"), rooms_db, messages_db);
    load_balancer.add_server(server1);


    //chatroom1.add_participant(user1.clone());
    //chatroom1.add_participant(user2.clone());


    
    let main_server = match load_balancer.get_server() {
        Some(server) => {server},
        None => {
            println!("Cannot get server");
            return Err(());
        },
    };


    let _ = main_server.add_user_to_room(user1.clone(), chatroom_id.clone());

    if let Ok(message) = Message::new(
        String::from("4567"),
        user1.get_username().clone(),
        chatroom_id.clone(),
        String::from("Hello!"),
    ) {
        let _ = main_server.send_message(message);
    }
    
    //println!("Chat Room:   id: {}, users count: {}, messages count: {}", chatroom_id.clone(), main_server.get_room_participants(chatroom_id.clone()).unwrap().len(), main_server.get_room_messages(chatroom_id.clone()).unwrap().len());

    
    let _ = main_server.add_user_to_room(user2.clone(), chatroom_id.clone());
    
    if let Ok(message) = Message::new(
        String::from("3948"),
        user2.get_username().clone(),
        chatroom_id.clone(),
        String::from("Hi!"),
    ) {
        let _ = main_server.send_message(message);
    }
    
    Ok(())
} */
