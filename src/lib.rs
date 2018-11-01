#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(feature = "nightly", deny(unsafe_code))]
//#![cfg_attr(test, deny(warnings))]

#[macro_use]
extern crate bson;
#[macro_use]
extern crate failure;
// TODO: return json
// #[macro_use]
// extern crate serde_derive;
//
// extern crate serde;
// extern crate serde_json;

use bson::{Bson, Document};

use std::string::String;
mod error;
pub use error::{Error, ErrorKind, Result};

// struct ParseOptions {
//   semantic_types: bool,
//   store_values: bool,
// }
mod schema;
use schema::DocumentKind;
use schema::Field;
use schema::MongoDBSchema;
use schema::PrimitiveType;

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

pub fn parser(_schema: &str) -> Result<MongoDBSchema> {
  let mut values_vec = Vec::new();
  values_vec.push(1);

  let primitive_type = PrimitiveType {
    name: String::from("Number"),
    path: String::from("path"),
    count: 1,
    probability: 0.75,
    unique: 1,
    has_duplicates: false,
    values: values_vec,
  };

  let primitive_type = DocumentKind::PrimitiveType(primitive_type);

  let mut types_vec = Vec::new();
  types_vec.push(primitive_type);

  let field = Field {
    name: String::from("_id"),
    path: String::from("path"),
    count: 1,
    field_type: String::from("Number"),
    probability: 0.75,
    has_duplicates: false,
    types: types_vec,
  };

  let mut field_vec = Vec::new();
  field_vec.push(field);

  let mongodb_schema = MongoDBSchema {
    count: 4,
    fields: field_vec,
  };

  // ideally want to pass mongodb schema struct to js land, but with Vecs in
  // the struct, it's currently not possible:
  // https://github.com/rustwasm/wasm-bindgen/issues/111
  // let ret = JsValue::from_serde(&mongodb_schema).unwrap();
  // Ok(ret)
  Ok(mongodb_schema)
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
