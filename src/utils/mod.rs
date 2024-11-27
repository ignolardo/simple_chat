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