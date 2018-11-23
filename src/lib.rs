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
pub struct SchemaParser<T> {
  count: i64,
  fields: Option<Vec<Field<T>>>,
}

#[derive(Serialize, Debug, Clone)]
struct Field<T> {
  name: String,
  path: String,
  count: usize,
  field_type: String,
  probability: Option<f64>,
  has_duplicates: Option<bool>,
  types: Option<Vec<FieldType<T>>>,
}

#[derive(Serialize, Debug, Clone)]
struct FieldType<T> {
  name: String,
  path: String,
  count: usize,
  probability: Option<f64>,
  values: Vec<T>,
  has_duplicates: Option<bool>,
  unique: Option<usize>,
}

impl SchemaParser {
  pub fn new() -> Self {
    SchemaParser {
      count: 0,
      fields: None,
    }
  }

  pub fn write(&self, json: &str) -> Result<()> {
    let val: Value = serde_json::from_str(json).unwrap();
    let bson = Bson::from(val);
    let doc = bson.as_document().unwrap().to_owned();
    let count = &self.count + 1;
    mem::replace(&mut self.count, count);
    let fields = self.generate_field(doc, &None);
    Ok(())
  }

  pub fn flush(&self) -> Option<&Document> {
    unimplemented!();
  }

  fn generate_field(&self, doc: Document, path: &Option<String>) -> &mut Self {
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

      let mut value_doc = doc! {
        "name": &key,
        "count": count,
        "path": &current_path,
      };

      let mut types = vec![];
      let value_type = self.add_to_types(value, current_path);

      if let Some(value_type) = value_type {
        types.push(bson::to_bson(&value_type).unwrap());
        value_doc.insert("types", types);
      }
      self.fields.push(value_doc);
    }
    self
  }

  fn add_to_types(&mut self, value: Bson, path: String) -> Option<Document> {
    match value {
      Bson::Document(subdoc) => {
        let doc = self.generate_field(subdoc, &Some(path));
        let bson = bson::to_bson(doc).unwrap();
        let bson_doc = bson.as_document();
        if let Some(bson_doc) = bson_doc {
          Some(bson_doc.to_owned())
        } else {
          None
        }
      }
      Bson::Array(_) => {
        let mut values = doc!{
          "path": &path,
        };
        let bson_type = &mut self.add_type(&value);
        if let Some(bson_type) = bson_type {
          values.insert("name", bson::to_bson(&bson_type).unwrap());
          values.insert("bsonType", bson::to_bson(&bson_type).unwrap());
        }
        // add values item in array as a separate func;
        values.insert("values", bson::to_bson(&value).unwrap());

        Some(values)
      }
      _ => {
        let mut values = doc!{
          "path": &path,
        };
        let bson_type = &mut self.add_type(&value);
        if let Some(bson_type) = bson_type {
          values.insert("name", bson::to_bson(&bson_type).unwrap());
          values.insert("bsonType", bson::to_bson(&bson_type).unwrap());
        }
        let val = vec![&value];
        values.insert("values", bson::to_bson(&val).unwrap());

        Some(values)
      }
    }
  }

  fn add_type(&mut self, value: &Bson) -> Option<&str> {
    match value {
      Bson::FloatingPoint(_) | Bson::I32(_) | Bson::I64(_) => Some("Number"),
      Bson::Document(_) => Some("Document"),
      Bson::Boolean(_) => Some("Boolean"),
      Bson::String(_) => Some("String"),
      Bson::Array(_) => Some("Array"),
      Bson::Null => Some("Null"),
      _ => None,
    }
  }
}

#[cfg(test)]
mod test {
  use super::SchemaParser;
  use std::fs;

  // #[test]
  // fn simple_schema_gen() {
  //   let doc = doc! {
  //     "_id": {
  //       "$oid": "50319491fe4dce143835c552"
  //     },
  //     "membership_status": "ACTIVE",
  //     "name": "Ellie J Clarke",
  //     "gender": "male",
  //     "age": 36,
  //     "phone_no": "+19786213180",
  //     "last_login": {
  //       "$date": "2014-01-31T22:26:33.000Z"
  //     },
  //     "address": {
  //       "city": "El Paso, Texas",
  //       "street": "133 Aloha Ave",
  //       "postal_code": 50017,
  //       "country": "USA",
  //       "location": {
  //         "type": "Point",
  //         "coordinates":[-73.4446279457308,40.89674015263909]
  //       }
  //     },
  //     "favorite_feature": "Auth",
  //     "email": "corefinder88@hotmail.com"
  //   };

  //   println!("{}", generate_schema_from_document(doc, None));
  // }

  #[test]
  fn json_file_gen() {
    let file = fs::read_to_string("examples/fanclub.json").unwrap();
    let file: Vec<&str> = file.split('\n').collect();
    let mut schema_parser = SchemaParser::new();
    for mut json in file {
      schema_parser.write(&json).unwrap();
    }
  }
}
