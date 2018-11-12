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
    add_type(&mut value_doc, value);
    add_to_type(&mut value_doc, value, current_path);

    fields.push(Bson::Document(value_doc));
  }

  doc! {
    "count": count as i64,
    "fields": fields
  }
}

fn add_type(doc: &mut Document, value: Bson) {
  let value_type = match value {
    Bson::FloatingPoint(_) | Bson::I32(_) | Bson::I64(_) => "Number",
    Bson::Document(_) => "Document",
    Bson::Boolean(_) => "Boolean",
    Bson::String(_) => "String",
    Bson::Array(_) => "Array",
    Bson::Null => "Null",
    _ => unimplemented!(),
  };

  doc.insert("type", value_type);
}

fn add_to_type(doc: &mut Document, value: Bson, path: String) {
  let doc_type = doc.get_str("type").unwrap();
  let schema = match doc_type {
    "Document" => generate_schema_from_document(value, Some(path)),
    "Array" => unimplemented!(),
    _ => unimplemented!(),
  };
  doc.insert("type", schema);
}

#[cfg(test)]
mod test {
  use super::generate_schema_from_document;

  #[test]
  fn simple_schema_gen() {
    let d = doc! {
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
          "coordinates": [ -106.3974968970265, 31.79689833641156 ]
        }
      },
      "favorite_feature": "Auth",
      "email": "corefinder88@hotmail.com"
    };

    println!("{}", generate_schema_from_document(d, None));
  }
}
