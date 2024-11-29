#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;

pub use chatroom::ChatRoom;
use clap::{Arg, Command};
pub use message::Message;
pub use user::User;
pub use chatserver::ChatServer;
pub use loadbalancer::LoadBalancer;

pub mod chatroom;
pub mod message;
pub mod user;
pub mod chatserver;
pub mod loadbalancer;
pub mod servers;
pub mod utils;


pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn build_command() -> Command {
      Command::new("Simple Chat App")
            .author("Ignacio Rivadero, rivaderonacho@gmail.com")
            .version(VERSION)
            .about("A simple chat app powered by Rocket.rs")
            .arg(
                  Arg::new("server")
                  .short('s')
                  .long("server")
                  .value_name("SERVER")
                  .help("Set the type of server to run")
            )
            .arg(
                  Arg::new("id")
                  .long("id")
                  .value_name("ID")
                  .help("Set the server id")
                  .required_if_eq("server", "messages")
            )
            .arg(
                  Arg::new("port")
                  .short('p')
                  .long("port")
                  .value_name("PORT")
                  .value_parser(clap::value_parser!(u16))
                  .help("Set the server port")
                  .required_if_eq("server", "messages")
            )
            .after_help(
                  "Longer explanation to appear after the options when \
                  displaying the help information from --help or -h"
            )
}
