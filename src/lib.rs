#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(feature = "nightly", deny(unsafe_code))]
#![cfg_attr(test, deny(warnings))]

#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

extern crate serde;
extern crate serde_json;

use std::string::String;

mod error;
pub use error::{Error, ErrorKind, Result};

struct ParseOptions {
  semantic_types: bool,
  store_values: bool,
}

mod schema;
use schema::DocumentKind;
use schema::Field;
use schema::MongoDBSchema;
use schema::PrimitiveType;

#[wasm_bindgen]
pub fn parser(_schema: &str) -> Result<String> {
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
  // return Ok(mongodb_schema);

  let json = serde_json::to_string(&mongodb_schema).unwrap();

  Ok(json)
}
