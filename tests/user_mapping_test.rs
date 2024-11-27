use std::net::SocketAddr;
use rocket::local::blocking::Client;
use simple_chat::servers::user_mapping::new_rocket;
use simple_chat::utils::addr;

#[test]
fn get_user_1234_server() {

      let id = "1234";
      let response = server_req(id);
      assert_eq!(response,String::from("\"0.1.2.3:1234\""))
}

#[test]
fn get_user_1267_server() {

      let id = "1267";
      let response = server_req(id);
      assert_eq!(response,String::from("\"0.1.2.4:1267\""))
}

#[test]
fn get_user_4590_server() {

      let id = "4590";
      let response = server_req(id);
      assert_eq!(response,String::from("\"0.1.2.5:4590\""))
}

#[test]
fn get_user_2846_server() {

      let id = "2846";
      let response = server_req(id);
      assert_eq!(response,String::from("\"0.1.2.6:2846\""))
}

#[test]
fn post_user_4567_server() {
      let addr = addr([0,0,0,0], 1111);
      let id = "1234";
      let response = put_server_req(id, addr);
      assert_eq!(response,"Done")
}


fn server_req(id: &str) -> String {
      let rocket = new_rocket();
      let response = Client::tracked(rocket)
            .unwrap()
            .get(format!("/{}",id).as_str())
            .dispatch()
            .into_string()
            .unwrap();
      response
}

fn put_server_req(id: &str, addr: SocketAddr) -> String
{    
      let addr = serde_json::to_string(&addr).unwrap();
      let addr = addr
            .replace(",", "%2C")
            .replace(":", "%3A")
            .replace("[", "%5B")
            .replace("]", "%5D");
      let route = format!("/{}/{}", id, addr).replace("\"", "");

      let rocket = new_rocket();
      let response = Client::tracked(rocket)
            .unwrap()
            .put(route.as_str())
            .dispatch()
            .into_string()
            .unwrap();
      response
}