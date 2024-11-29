use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use rocket::response::status::BadRequest;
use rocket::response::status::Custom;
use rocket::Build;
use rocket::Rocket;
use serde::Serialize;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::utils;
use crate::utils::addr;

pub struct UserMappingServer {
      id: String,
      addr: SocketAddr,
}

impl UserMappingServer {
      pub fn new(id: String, addr: SocketAddr) -> Self {
            Self {
                  id,
                  addr,
            }
      }

      pub async fn init(&self) {
            env::set_var("ROCKET_PORT", self.addr.port().to_string());
            let _ = rocket::build()
                  .mount("/", routes![get_user_server_addr, set_user_server_addr])
                  .manage(Mutex::new(HashMap::<String, SocketAddr>::new()))
                  .launch()
                  .await;
      }
}

#[derive(Serialize, Deserialize)]
pub struct SetUserServerBody<'r> {
      pub id: &'r str,
      pub addr: SocketAddr,
}

#[get("/<id>")]
async fn get_user_server_addr(users: &rocket::State<Mutex<HashMap<String, SocketAddr>>>, id: &str) -> String {
      let users = users.lock().await;
      if let Some(addr) = users.get(&String::from(id)) {
            return serde_json::to_string(addr).expect("No valid Socket Address");
      }
      String::from("")
}

#[put("/<id>/<addr>")]
async fn set_user_server_addr(
      users: &rocket::State<Mutex<HashMap<String, SocketAddr>>>, 
      id: &str,
      addr: &str,
)
-> Result<rocket::response::status::Custom<String>,rocket::response::status::BadRequest<String>>
{     
      let result = utils::string_to_addr(addr);
      if result.is_err() {
            return Err(BadRequest(String::from("Address not valid")));
      }
      let addr = result.unwrap();

      let mut users = users.lock().await;
      users.insert(String::from(id), addr);

      Ok(Custom(rocket::http::Status::Ok, String::from("Done")))
}

pub fn new_rocket() -> Rocket<Build> {
      let mut map = HashMap::<String, SocketAddr>::new();
      map.insert(String::from("1234"), addr([0,1,2,3], 1234));
      //{"ip":[0,1,2,3],"port":1234}
      map.insert(String::from("1267"), addr([0,1,2,4], 1267));
      //{"ip":[0,1,2,4],"port":1267}
      map.insert(String::from("4590"), addr([0,1,2,5], 4590));
      //{"ip":[0,1,2,5],"port":4590}
      map.insert(String::from("2846"), addr([0,1,2,6], 2846));
      //{"ip":[0,1,2,6],"port":2846}

      rocket::build()
            .mount("/", routes![get_user_server_addr, set_user_server_addr])
            .manage(Mutex::new(map))
}