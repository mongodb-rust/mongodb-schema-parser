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

fn add_field_schema_to_document(doc: &mut Document, value: Bson, path: String) {
  let value_type = match value {
    Bson::FloatingPoint(_) | Bson::I32(_) | Bson::I64(_) => "Number",
    Bson::String(_) => "String",
    Bson::Boolean(_) => "Boolean",
    Bson::Document(subdoc) => {
      let schema = generate_schema_from_document(subdoc, Some(path));
      doc.insert("type", schema);
      return;
    }
    Bson::Array(_) => "Array",
    Bson::Null => "Null",
    _ => unimplemented!(),
  };

  doc.insert("type", value_type);
}

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
      "path": current_path,
    };
    add_field_schema_to_document(&mut value_doc, value, key);

    fields.push(Bson::Document(value_doc));
  }

  doc! {
    // NOTE: This will be incorrect if the number of fields is greater than i64::MAX
    "count": count as i64,
    "fields": fields
  }
}

#[cfg(test)]
mod test {
  use super::generate_schema_from_document;
  use bson::Bson;

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
