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
use failure::Error;
use std::mem;
use std::string::String;

// mod error;
// pub use error::{Error, ErrorKind, Result};
/// Custom Result type

pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaParser {
  count: i64,
  fields: Vec<Field>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Field {
  name: String,
  path: String,
  count: usize,
  field_type: Option<String>,
  probability: Option<f64>,
  has_duplicates: Option<bool>,
  types: Vec<FieldType>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
enum ValueType {
  Str(String),
  I32(i32),
  I64(i64),
  FloatingPoint(f64),
  Boolean(bool),
}

impl Field {
  fn new(name: String, path: String, count: usize) -> Self {
    Field {
      name: name.clone(),
      count: count,
      path: path,
      field_type: None,
      probability: None,
      has_duplicates: None,
      types: Vec::new(),
    }
  }

  fn get_path(name: String, path: &Option<String>) -> String {
    match &path {
      None => name.clone(),
      Some(path) => {
        let mut path = path.clone();
        path.push_str(".");
        path.push_str(&name);
        path
      }
    }
  }
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

  fn add_to_type(
    mut self,
    value: &Bson,
    mut value_vec: Vec<ValueType>,
  ) -> Option<Self> {
    let bson_value = value.clone();
    match value {
      Bson::Array(arr) => {
        let bson_type = Self::get_type(&bson_value);
        self.set_name(bson_type.clone());
        self.set_bson_type(bson_type.clone());
        // add values item in array as a separate func;
        for val in arr.iter() {
          let value_type = Self::get_value(val);

          if let Some(value_type) = value_type {
            value_vec.push(value_type)
          }
        }
        self.set_values(value_vec);
        Some(self)
      }
      _ => {
        let value_type = Self::get_value(&bson_value);
        let bson_type = Self::get_type(&bson_value);
        self.set_name(bson_type.clone());
        self.set_bson_type(bson_type.clone());
        // add values item in array as a separate func;
        if let Some(value_type) = value_type {
          value_vec.push(value_type);
          self.set_values(value_vec);
        }
        Some(self)
      }
    }
  }

  fn get_value(value: &Bson) -> Option<ValueType> {
    match value {
      Bson::FloatingPoint(num) => Some(ValueType::FloatingPoint(*num)),
      Bson::Boolean(boolean) => Some(ValueType::Boolean(*boolean)),
      Bson::String(string) => Some(ValueType::Str(string.to_string())),
      Bson::I32(num) => Some(ValueType::I32(*num)),
      Bson::I64(num) => Some(ValueType::I64(*num)),
      _ => None,
    }
  }

  fn get_type(value: &Bson) -> Option<String> {
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

  fn push_value(&mut self, value: ValueType) {
    self.values.push(value)
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
    let val: Value = serde_json::from_str(json)?;
    let bson = Bson::from(val);
    // should do a match for NoneError
    let doc = bson.as_document().unwrap().to_owned();
    let count = &self.count + 1;
    mem::replace(&mut self.count, count);
    self.generate_field(doc, &None);
    Ok(())
  }

  pub fn to_json(self) -> Result<String> {
    Ok(serde_json::to_string(&self)?)
  }

  fn generate_field(&mut self, doc: Document, path: &Option<String>) {
    let count = 0;

    for (key, value) in doc {
      // check if we already have a field for this key;
      // this check should also be checking for uniqueness
      // 'inner:
      for mut field in &mut self.fields {
        if field.name == key {
          // need to seet count here as well
          println!("key: {}", field.name);
          for mut field_type in &mut field.types {
            let field_count = field_type.count + 1;
            field_type.set_count(field_count);
            let value_type = FieldType::get_value(&value);
            if let Some(value_type) = value_type {
              field_type.push_value(value_type);
            }
          }
        // break 'inner;
        } else {
          let current_path = Field::get_path(key.clone(), path);
          let mut field = Field::new(key.clone(), &current_path, count);
          let mut value_vec = Vec::new();

          match &value {
            Bson::Document(subdoc) => {
              self.generate_field(subdoc.to_owned(), &Some(current_path));
            }
            _ => {
              let field_type =
                FieldType::new(current_path).add_to_type(&value, value_vec);
              if let Some(field_type) = field_type {
                field.types.push(field_type.to_owned());
              }
            }
          };
          self.fields.push(field);
        }
      }
    }
  }
}

#[cfg(test)]
mod test {
  use super::SchemaParser;
  use failure::Error;
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

    schema_parser.to_json();
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
    // println!("{:?}", schema);
    Ok(())
  }
}
