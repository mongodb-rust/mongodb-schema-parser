// use failure::Error;
// use mongodb_schema_parser::SchemaParser;
// use mongodb::{Client, options::ClientOptions, options::FindOptions};
// use bson::{doc, bson};
// use std::fs;

// #[test]
// fn json_file_fanclub() -> Result<(), Error> {
//   // TODO: check timing on running this test
//   let file = fs::read_to_string("fixtures/fanclub.json")?;
//   let vec: Vec<&str> = file.trim().split('\n').collect();
//   let mut schema_parser = SchemaParser::new();
//   for json in vec {
//     // this panics i think ?
//     schema_parser.write_json(&json)?;
//   }
//   let schema = schema_parser.into_json();
//   println!("{:?}", schema);
//   Ok(())
// }

// // connect to an actual mongodb collection
// #[test]
// fn test_mongodb_collection() -> Result<(), Error> {

// }

// #[test]
// fn binary_file_sales_supplies() -> Result<(), Error> {
//   // TODO: check timing on running this test
//   let file = fs::read("fixtures/sales_data.json")?;
//   // let vec: Vec<&str> = file.trim().split('\n').collect();
//   let mut schema_parser = SchemaParser::new();
//   for item in file {
//     // this panics i think ?
//     schema_parser.write_raw(item)?;
//   }
//   let schema = schema_parser.into_json();
//   println!("{:?}", schema);
//   Ok(())
// }