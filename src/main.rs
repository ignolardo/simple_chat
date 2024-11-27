use simple_chat::utils::addr;

#[tokio::main]
async fn main()
{
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let _ = simple_chat::servers::MessageServer::new(
        String::from("1223"), 
        addr([0,0,0,0], 8000)
    ).init().await;
}