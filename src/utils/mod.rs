use std::net::SocketAddr;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::sync::Mutex;

pub mod thread_pool;
pub mod channels;

pub type ArcMut<T> = Arc<Mutex<T>>;

pub fn new_arcmut<T>(value: T) -> ArcMut<T> {
      Arc::new(Mutex::new(value))
}



pub type Tx<T> = tokio::sync::mpsc::UnboundedSender<T>;
pub type Rx<T> = tokio::sync::mpsc::UnboundedReceiver<T>;

pub fn addr(ip: [u8;4], port: u16) -> SocketAddr {
      SocketAddr::new(IpAddr::V4(Ipv4Addr::new(
            *ip.get(0).unwrap(),
            *ip.get(1).unwrap(),
            *ip.get(2).unwrap(),
            *ip.get(3).unwrap())), port)
}

/*
      The address taken must not have "" characters
 */
pub fn string_to_addr(addr: &str) -> Result<SocketAddr,serde_json::Error> {
      serde_json::from_str::<SocketAddr>(format!("\"{}\"",addr).as_str())
}

pub fn addr_to_url_safe(addr: SocketAddr) -> String {
      let addr = serde_json::to_string(&addr).unwrap();
      let addr = addr
            .replace(",", "%2C")
            .replace(":", "%3A")
            .replace("[", "%5B")
            .replace("]", "%5D")
            .replace("\"", "");
      addr
}

pub fn url_safe_to_addr(addr: String) -> Result<SocketAddr,serde_json::Error> {
      let addr = addr
            .replace("%2C", ",")
            .replace("%3A", ":")
            .replace("%5B", "[")
            .replace("%5D", "]");
      let result: Result<SocketAddr,serde_json::Error> = serde_json::from_str(addr.as_str());
      result
}

pub async fn init_message_server(id: String, port: u16) {
      let _ = crate::servers::MessageServer::new(
            id, 
            addr([0,0,0,0], port)
      ).init().await;
}

pub async fn init_user_mapping_server(id: String, port: u16) {
      let _ = crate::servers::UserMappingServer::new(id,addr([0,0,0,0], port)).init().await;
}