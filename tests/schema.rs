use failure::Error;
use mongodb_schema_parser::SchemaParser;
use std::fs;

static DOC: &str = r#"{
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

#[test]
fn it_creates_correct_number_of_fields() {
  let mut schema_parser = SchemaParser::new();
  schema_parser.write(DOC).unwrap();
  assert_eq!(schema_parser.fields.len(), 10);
}

#[test]
fn it_combines_arrays_for_same_field_into_same_types_vector() {
  let mut schema_parser = SchemaParser::new();
  let vec_json1 = r#"{"animals": ["cat", "dog"]}"#;
  let vec_json2 = r#"{"animals": ["wallaby", "bird"]}"#;
  schema_parser.write(vec_json1).unwrap();
  schema_parser.write(vec_json2).unwrap();
  assert_eq!(schema_parser.fields.len(), 1);
  let field = schema_parser.fields.get("animals");
  if let Some(field) = field {
    assert_eq!(field.types.len(), 1);
    let field_type = field.types.get("Array");
    if let Some(field_type) = field_type {
      assert_eq!(field_type.values.len(), 4);
    }
  }
}

#[test]
fn it_creates_different_field_types() {
  let mut schema_parser = SchemaParser::new();
  let number_json = r#"{"phone_number": 491234568789}"#;
  let string_json = r#"{"phone_number": "+441234456789"}"#;
  schema_parser.write(number_json).unwrap();
  schema_parser.write(string_json).unwrap();
  let field = schema_parser.fields.get("phone_number");
  if let Some(field) = field {
    assert_eq!(field.count, 2);
    assert_eq!(field.bson_types.len(), 2);
    assert_eq!(field.types.len(), 2);
  }
}

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
