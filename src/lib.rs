#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(feature = "nightly", deny(unsafe_code))]
//#![cfg_attr(test, deny(warnings))]

#[macro_use]
extern crate bson;
extern crate failure;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

use serde_json::Value;

use bson::{Bson, Document};
use std::mem;
use std::string::String;

mod error;
pub use error::{Error, ErrorKind, Result};

#[derive(Serialize, Debug, Clone)]
pub struct SchemaParser {
  count: i64,
  fields: Vec<Field>,
}

#[derive(Serialize, Debug, Clone)]
struct Field {
  name: String,
  path: String,
  count: usize,
  field_type: Option<String>,
  probability: Option<f64>,
  has_duplicates: Option<bool>,
  types: Vec<FieldType>,
}

#[derive(Serialize, Debug, Clone)]
struct FieldType {
  name: Option<String>,
  path: String,
  count: usize,
  bsonType: Option<String>,
  probability: Option<f64>,
  values: Vec<ValueType>,
  has_duplicates: Option<bool>,
  unique: Option<usize>,
}

#[derive(Serialize, Debug, Clone)]
enum ValueType {
  Str(String),
  I32(i32),
  I64(i64),
  FloatingPoint(f64),
  Boolean(bool),
}

impl FieldType {
  fn new(path: String) -> Self {
    FieldType {
      name: None,
      path: path,
      bsonType: None,
      count: 0,
      probability: None,
      values: Vec::new(),
      has_duplicates: None,
      unique: None,
    }
  }

  fn set_name(&mut self, name: Option<String>) {
    self.name = name
  }

  fn set_bson_type(&mut self, bsontype: Option<String>) {
    self.bsonType = bsontype
  }

  fn set_count(&mut self, count: usize) {
    self.count = count
  }

  fn set_values(&mut self, values: Vec<ValueType>) {
    self.values = values
  }
}

impl SchemaParser {
  pub fn new() -> Self {
    SchemaParser {
      count: 0,
      fields: Vec::new(),
    }
  }

  pub fn write(&mut self, json: &str) -> Result<()> {
    let val: Value = serde_json::from_str(json).unwrap();
    let bson = Bson::from(val);
    let doc = bson.as_document().unwrap().to_owned();
    let count = &self.count + 1;
    mem::replace(&mut self.count, count);
    self.generate_field(doc, &None);
    Ok(())
  }

  pub fn flush(self) -> Self {
    self
  }

  fn generate_field(&mut self, doc: Document, path: &Option<String>) -> &Self {
    let count = 0;

    for (key, value) in doc {
      let current_path = match &path {
        None => key.clone(),
        Some(path) => {
          let mut path = path.clone();
          path.push_str(".");
          path.push_str(&key);
          path
        }
      };

      let mut field = Field {
        name: key.clone(),
        count: count,
        path: current_path.clone(),
        field_type: None,
        probability: None,
        has_duplicates: None,
        types: Vec::new(),
      };

      let field_type = &self.add_to_types(value, current_path);
      if let Some(field_type) = field_type {
        field.types.push(field_type.to_owned());
      }
      self.fields.push(field);
    }
    self
  }

  fn add_to_types(&mut self, value: Bson, path: String) -> Option<FieldType> {
    let bson_value = value.clone();
    match value {
      Bson::Document(subdoc) => {
        self.generate_field(subdoc, &Some(path));
        None
      }
      Bson::Array(arr) => {
        let mut field_type = FieldType::new(path.clone());
        let bson_type = self.set_type(&bson_value);
        field_type.set_name(bson_type.clone());
        field_type.set_bson_type(bson_type.clone());
        // add values item in array as a separate func;
        let mut value_type_vec = Vec::new();
        for val in arr.iter() {
          let value_type = self.get_type(val);

          if let Some(value_type) = value_type {
            value_type_vec.push(value_type)
          }
        }
        field_type.set_values(value_type_vec);
        Some(field_type)
      }
      _ => {
        let mut field_type = FieldType::new(path.clone());
        let value_type = self.get_type(&bson_value);
        let bson_type = self.set_type(&bson_value);
        field_type.set_name(bson_type.clone());
        field_type.set_bson_type(bson_type.clone());
        // add values item in array as a separate func;
        if let Some(value_type) = value_type {
          field_type.set_values(vec![value_type]);
        }
        Some(field_type)
      }
    }
  }

  fn get_type(&self, value: &Bson) -> Option<ValueType> {
    match value {
      Bson::FloatingPoint(num) => Some(ValueType::FloatingPoint(*num)),
      Bson::Boolean(boolean) => Some(ValueType::Boolean(*boolean)),
      Bson::String(string) => Some(ValueType::Str(string.to_string())),
      Bson::I32(num) => Some(ValueType::I32(*num)),
      Bson::I64(num) => Some(ValueType::I64(*num)),
      _ => None,
    }
  }

  fn set_type(&mut self, value: &Bson) -> Option<String> {
    match value {
      Bson::FloatingPoint(_) | Bson::I32(_) | Bson::I64(_) => {
        Some(String::from("Number"))
      }
      Bson::Document(_) => Some(String::from("Document")),
      Bson::Boolean(_) => Some(String::from("Boolean")),
      Bson::String(_) => Some(String::from("String")),
      Bson::Array(_) => Some(String::from("Array")),
      Bson::Null => Some(String::from("Null")),
      _ => None,
    }
  }
}

#[cfg(test)]
mod test {
  use super::SchemaParser;
  use std::fs;

  #[test]
  fn simple_schema_gen() {
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
    schema_parser.write(doc).unwrap();

    let schema = schema_parser.flush();

    println!("{:?}", schema);
  }

  #[test]
  fn json_file_gen() {
    let file = fs::read_to_string("examples/fanclub.json").unwrap();
    let file: Vec<&str> = file.split('\n').collect();
    let mut schema_parser = SchemaParser::new();
    for mut json in file {
      schema_parser.write(&json).unwrap();
    }
    let schema = schema_parser.flush();
    println!("{:?}", schema)
  }
}
