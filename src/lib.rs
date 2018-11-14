#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(feature = "nightly", deny(unsafe_code))]
//#![cfg_attr(test, deny(warnings))]

#[macro_use]
extern crate bson;
#[macro_use]
extern crate failure;

// there is an attribute for test only imports
use bson::{Bson, Document};

use std::string::String;
mod error;
pub use error::{Error, ErrorKind, Result};

pub fn generate_schema_from_document(
  doc: Document,
  path: Option<String>,
) -> Document {
  let count = doc.len();

  let mut fields = vec![];

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
      "path": &current_path,
    };

    let mut types = vec![];

    let value_type = add_to_types(value, current_path);

    if let Some(value_type) = value_type {
      types.push(bson::to_bson(&value_type).unwrap());
      value_doc.insert("types", types);
    }

    fields.push(Bson::Document(value_doc));
  }

  doc! {
    "count": count as i64,
    "fields": fields
  }
}

fn add_type(value: &Bson) -> Option<&str> {
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

fn add_to_types(value: Bson, path: String) -> Option<Document> {
  let bson_type = add_type(&value);
  let match_value = value.clone();
  match match_value {
    Bson::Document(subdoc) => {
      Some(generate_schema_from_document(subdoc, Some(path)))
    }
    Bson::Array(_) => {
      let mut values = doc!{
        "path": &path,
      };
      if let Some(bson_type) = bson_type {
        values.insert("name", bson::to_bson(&bson_type).unwrap());
        values.insert("bsonType", bson::to_bson(&bson_type).unwrap());
      }
      values.insert("values", bson::to_bson(&value).unwrap());

      Some(values)
    }
    _ => {
      let mut values = doc!{
        "path": &path,
      };
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

#[cfg(test)]
mod test {
  use super::generate_schema_from_document;
  use super::Bson;

  extern crate serde;
  extern crate serde_json;

  #[test]
  fn simple_schema_gen() {
    let doc = doc! {
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
    };

    println!("{}", generate_schema_from_document(doc, None));
  }

  fn json_file_gen() {
    let mut json: serde_json::Value =
      serde_json::from_str("../examples/fanclub.json")
        .expect("JSON File not well formatted");

    let doc = Bson::from_json(json);
    println!("{}", generate_schema_from_document(doc, None))
  }
}
