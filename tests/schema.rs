use failure::Error;
use mongodb_schema_parser::SchemaParser;
use std::fs;

#[test]
fn json_file_gen() -> Result<(), Error> {
  // TODO: check timing on running this test
  let file = fs::read_to_string("examples/fanclub.json")?;
  let vec: Vec<&str> = file.trim().split('\n').collect();
  let mut schema_parser = SchemaParser::new();
  for json in vec {
    // this panics i think ?
    schema_parser.write(&json)?;
  }
  let schema = schema_parser.to_json();
  println!("{:?}", schema);
  Ok(())
}
