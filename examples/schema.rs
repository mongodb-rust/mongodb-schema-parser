// needs to not have crate-type = ["cdylib"] in cargo.toml
// extern crate mongodb_schema_parser;
// use mongodb_schema_parser::SchemaParser;
//
// use std::fs;
//
// pub fn main() {
//   let mut file = fs::read_to_string("./fanclub.json").unwrap();
//   let file: Vec<&str> = file.split("\n").collect();
//   let schema_parser = SchemaParser::new();
//   for json in file {
//     schema_parser.write(&json).unwrap();
//   }
//   let result = schema_parser.to_json();
//   println!("{:?}", result)
// }

fn main() {
  unimplemented!();
}
