extern crate failure;
extern crate mongodb_schema_parser;

use failure::Error;
use mongodb_schema_parser::SchemaParser;
use std::fs;

#[test]
fn simple_schema_gen() -> Result<(), Error> {
  let doc = r#"{
    "_id": {
      "$oid": "50319491fe4dce143835c552"
    },
    "membership_status": "ACTIVE",
    "name": "Ellie J Clarke",
    "gender": "male",
    "age": 36,
    "phone_no": "+19786213180",
    "last_login": {
      "$date": "2014-01-31T22:26:33.000Z"
    },
    "address": {
      "city": "El Paso, Texas",
      "street": "133 Aloha Ave",
      "postal_code": 50017,
      "country": "USA",
      "location": {
        "type": "Point",
        "coordinates":[-73.4446279457308,40.89674015263909]
      }
    },
    "favorite_feature": "Auth",
    "email": "corefinder88@hotmail.com"
  }"#;

  let mut schema_parser = SchemaParser::new();
  schema_parser.write(doc)?;

  let schema = schema_parser.to_json();
  println!("{:?}", schema);
  Ok(())
}

#[test]
fn json_file_gen() -> Result<(), Error> {
  // TODO: check timing on running this test
  let file = fs::read_to_string("examples/fanclub.json")?;
  let vec: Vec<&str> = file.trim().split('\n').collect();
  let mut schema_parser = SchemaParser::new();
  for mut json in vec {
    // this panics i think ?
    schema_parser.write(&json)?;
  }
  let schema = schema_parser.to_json();
  println!("{:?}", schema);
  Ok(())
}
