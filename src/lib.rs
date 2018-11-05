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

fn add_field_schema_to_document(doc: &mut Document, value: Bson) {
  let value_type = match value {
    Bson::FloatingPoint(_) | Bson::I32(_) | Bson::I64(_) => "Number",
    Bson::Boolean(_) => "Boolean",
    Bson::Document(subdoc) => {
      let schema = generate_schema_from_document(subdoc);
      doc.insert("type", schema);
      return;
    }
    Bson::Array(arr) => "Array",
    Bson::Null => "Null",
    _ => unimplemented!(),
  };

  doc.insert("type", value_type);
}

fn generate_schema_from_document(doc: Document) -> Document {
  let count = doc.len();

  let fields = doc
    .into_iter()
    .fold(Vec::new(), |mut fields, (key, value)| {
      let mut value_doc = doc! {
        "name": key
      };
      add_field_schema_to_document(&mut value_doc, value);

      fields.push(Bson::Document(value_doc));
      fields
    });

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
      "foo": 12,
      "bar": [true, Bson::Null],
      "sub": {
        "x": -10
      }
    };

    println!("{}", generate_schema_from_document(d));
  }
}
