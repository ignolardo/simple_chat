use clap::ArgMatches;
use simple_chat::utils::init_message_server;
use simple_chat::utils::init_user_mapping_server;
use simple_chat::build_command;

#[tokio::main]
async fn main()
{
    simple_logger::SimpleLogger::new().env().init().unwrap();

    let matches = build_command().get_matches();

    let server: &String = matches.get_one("server").expect("Server type is required.");

    match server.as_str() {
        "messages" => {
            let (id,port) = get_id_and_port(&matches);
            init_message_server(id, port).await;
        },
        "user_mapping" => {
            let (id,port) = get_id_and_port(&matches);
            init_user_mapping_server(id, port).await;
        }
        _ => (),
    };
}

fn get_id_and_port(matches: &ArgMatches) -> (String,u16) {
    let id: &String = matches.get_one("id").expect("This server needs an id");
    let port: &u16 = matches.get_one("port").expect("This server needs a port");
    (id.clone(),*port)
}