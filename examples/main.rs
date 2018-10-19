#[macro_use]
extern crate mongodb;
extern crate bson;

use bson::{to_bson, Bson, Document};
use mongodb::db::ThreadedDatabase;
use mongodb::{Client, ThreadedClient};

pub fn main() {
  let client =
    Client::connect("localhost", 27017).expect("Failed to initialize client.");

  let coll = client.db("crunchbase").collection("companies");
  let cursor = coll.find(None, None).unwrap();
  for result in cursor {
    if let Ok(item) = result {
      if let Some(&Bson::String(ref name)) = item.get("name") {
        println!("company name: {}", name);
      }
    }
  }
}
