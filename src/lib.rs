//! #Infer a probabilistic schema for a MongoDB collection.
//! This crate creates a probabilistic scehma given a json-style string
//! representing a MongoDB collection. It can be used in both rust and javascript
//! given a WASM compilation.
//!
//! ## Usage: in Rust
//! ```rust
//! extern crate mongodb_schema_parser;
//! use mongodb_schema_parser::SchemaParser;
//! use std::fs;
//!
//! pub fn main () {
//!   let mut file = fs::read_to_string("examples/fanclub.json").unwrap();
//!   let file: Vec<&str> = file.trim().split("\n").collect();
//!   let mut schema_parser = SchemaParser::new();
//!   for json in file {
//!     schema_parser.write(&json).unwrap();
//!   }
//!   let result = schema_parser.read();
//! }
//! ```
//!
//! ## Usage: in JavaScript
//! Make sure your environment is setup for Web Assembly usage.
//! ```js
//! import { SchemaParser } from "mongodb-schema-parser"
//!
//! const schemaParser = new SchemaParser()
//!
//! // get the json file
//! fetch('./fanclub.json')
//!   .then(response => response.text())
//!   .then(data => {
//!     var json = data.split("\n")
//!     for (var i = 0; i < json.length; i++) {
//!       if (json[i] !== '') {
//!         // feed the parser json line by line
//!         schemaParser.write(json[i])
//!       }
//!     }
//!     // get the result as a json string
//!     var result = schemaParser.toJson()
//!     console.log(result)
//!   })
//! ```

#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(feature = "nightly", deny(unsafe_code))]
#![allow(clippy::new_without_default_derive)]
#![feature(test)]
//#![cfg_attr(test, deny(warnings))]

extern crate failure;
extern crate test;

#[macro_use]
extern crate bson;
use bson::{Bson, Document};

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
use serde_json::Value;

use wasm_bindgen::prelude::*;

// using custom allocator which is built specifically for wasm; makes it smaller
// + faster
extern crate wee_alloc;
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use std::collections::HashMap;
use std::string::String;

mod field;
use crate::field::Field;

mod field_type;
use crate::field_type::FieldType;

mod value_type;
use crate::value_type::ValueType;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SchemaParser {
  pub count: usize,
  pub fields: HashMap<String, Field>,
}

// Need to wrap schema parser impl for wasm suppport.
// Here we are wrapping the exported to JS land methods and mathing on Result to
// turn the error message to JsValue.
#[wasm_bindgen]
impl SchemaParser {
  /// Wrapper method for `SchemaParser::new()` to be used in JavaScript.
  /// `wasm_bindgen(js_name = "new")`
  #[wasm_bindgen(constructor)]
  #[wasm_bindgen(js_name = "new")]
  pub fn wasm_new() -> Self {
    Self::new()
  }

  /// Wrapper method for `schema_parser.write()` to be used in JavaScript.
  /// `wasm_bindgen(js_name = "write")`
  #[wasm_bindgen(js_name = "write")]
  pub fn wasm_write(&mut self, json: &str) -> Result<(), JsValue> {
    match self.write(json) {
      Err(e) => Err(JsValue::from_str(&format!("{}", e))),
      _ => Ok(()),
    }
  }

  /// Wrapper method for `schema_parser.to_json()` to be used in JavaScript.
  /// `wasm_bindgen(js_name = "toJson")`
  #[wasm_bindgen(js_name = "toJson")]
  pub fn wasm_to_json(&mut self) -> Result<JsValue, JsValue> {
    match JsValue::from_serde(&self) {
      Err(e) => Err(JsValue::from_str(&format!("{}", e))),
      Ok(val) => Ok(val),
    }
  }
}

impl SchemaParser {
  /// Returns a new instance of Schema Parser populated with zero `count` and an
  /// empty `fields` vector.
  ///
  /// # Examples
  /// ```
  /// use mongodb_schema_parser::SchemaParser;
  /// let schema_parser = SchemaParser::new();
  /// ```
  #[inline]
  pub fn new() -> Self {
    SchemaParser {
      count: 0,
      fields: HashMap::new(),
    }
  }

  /// Writes json-like string slices SchemaParser's fields vector.
  ///
  /// # Arguments
  /// * `json` - A json-like string slice. i.e { "name": "Nori", "type": "Cat"}
  ///
  /// # Examples
  /// ```
  /// use mongodb_schema_parser::SchemaParser;
  /// let mut schema_parser = SchemaParser::new();
  /// let json = r#"{ "name": "Chashu", "type": "Cat" }"#;
  /// schema_parser.write(&json);
  /// ```
  #[inline]
  pub fn write(&mut self, json: &str) -> Result<(), failure::Error> {
    let val: Value = serde_json::from_str(json)?;
    let bson = Bson::from(val);
    // should do a match for NoneError
    let doc = bson.as_document().unwrap().to_owned();
    self.update_count();
    self.generate_field(doc, &None);
    Ok(())
  }

  /// Finalizes and returns SchemaParser struct -- result of all parsed
  /// documents.
  ///
  /// # Examples
  /// ```
  /// use mongodb_schema_parser::SchemaParser;
  /// let mut schema_parser = SchemaParser::new();
  /// let json = r#"{ "name": "Chashu", "type": "Cat" }"#;
  /// schema_parser.write(&json);
  /// let schema = schema_parser.read();
  /// println!("{:?}", schema);
  /// ```
  // might want to rename this to finalize depending on how much manipulation
  // will have to be done later on.
  pub fn read(self) -> SchemaParser {
    self
  }

  /// Returns a serde_json string. This should be called after all values were
  /// written. This is also the result of the parsed documents.
  ///
  /// # Examples
  /// ```
  /// use mongodb_schema_parser::SchemaParser;
  /// let mut schema_parser = SchemaParser::new();
  /// let json = r#"{ "name": "Chashu", "type": "Cat" }"#;
  /// schema_parser.write(&json);
  /// let schema = schema_parser.to_json().unwrap();
  /// println!("{}", schema);
  /// ```
  #[inline]
  pub fn to_json(&self) -> Result<String, failure::Error> {
    Ok(serde_json::to_string(&self)?)
  }

  #[inline]
  fn generate_field(&mut self, doc: Document, path: &Option<String>) {
    for (key, value) in doc {
      self.update_or_create_field(
        key.to_string(),
        &value,
        &Field::get_path(key.to_string(), path),
      )
    }
  }

  #[inline]
  fn update_or_create_field(&mut self, key: String, value: &Bson, path: &str) {
    // check if we already have a field for this key;
    // if name exist, call self.update_field, otherwise create new
    if self.fields.contains_key(&key) {
      self.update_field(&key, &value);
    } else {
      let mut field = Field::new(key, &path);
      field.create_type(&value);
      self.fields.insert(field.name.to_string(), field);
    }
  }

  #[inline]
  fn update_field(&mut self, key: &str, value: &Bson) {
    let field = self.fields.get_mut(key);
    if let Some(field) = field {
      let mut has_duplicates = false;
      field.update_count();
      if !field.does_field_type_exist(&value) {
        // field type doesn't exist in field.types, create a new field_type
        field.create_type(&value);
      } else {
        let field_type = field.types.get_mut(&FieldType::get_type(&value));
        if let Some(field_type) = field_type {
          field_type.update_type(field.count, &value);
          has_duplicates = field_type.get_duplicates();
        }
      }
      field.set_duplicates(has_duplicates);
      field.set_probability(self.count);
    }
  }

  #[inline]
  fn update_count(&mut self) {
    self.count += 1
  }
}

#[cfg(test)]
mod tests {
  use self::test::Bencher;
  use super::*;

  #[test]
  fn it_creates_new() {
    let schema_parser = SchemaParser::new();
    assert_eq!(schema_parser.count, 0);
    assert_eq!(schema_parser.fields.len(), 0);
  }

  #[bench]
  fn bench_it_creates_new(bench: &mut Bencher) {
    bench.iter(|| SchemaParser::new);
  }

  #[test]
  fn it_writes() {
    let mut schema_parser = SchemaParser::new();
    let json_str = r#"{"name": "Nori", "type": "Cat"}"#;
    schema_parser.write(&json_str).unwrap();
    assert_eq!(schema_parser.count, 1);
    assert_eq!(schema_parser.fields.len(), 2);
  }

  #[bench]
  fn bench_it_writes(bench: &mut Bencher) {
    let mut schema_parser = SchemaParser::new();
    let json_str = r#"{"name": "Nori", "type": "Cat"}"#;
    bench.iter(|| schema_parser.write(&json_str));
  }

  // since read() only returns self right now, the test is the same as
  // it_writes()
  #[test]
  fn it_reads() {
    let mut schema_parser = SchemaParser::new();
    let json_str = r#"{"name": "Nori", "type": "Cat"}"#;
    schema_parser.write(&json_str).unwrap();
    let output = schema_parser.read();
    assert_eq!(output.count, 1);
    assert_eq!(output.fields.len(), 2);
  }

  #[test]
  fn it_formats_to_json() {
    let mut schema_parser = SchemaParser::new();
    let json_str = r#"{"name": "Chashu", "type": "Cat"}"#;
    schema_parser.write(&json_str).unwrap();
    let output = schema_parser.to_json().unwrap();
    assert_eq!(output, "{\"count\":1,\"fields\":{\"type\":{\"name\":\"type\",\"path\":\"type\",\"count\":1,\"bson_types\":[\"String\"],\"probability\":0.0,\"has_duplicates\":false,\"types\":{\"String\":{\"path\":\"type\",\"count\":1,\"bson_type\":\"String\",\"probability\":1.0,\"values\":[\"Cat\"],\"has_duplicates\":false,\"schema\":null,\"unique\":null}}},\"name\":{\"name\":\"name\",\"path\":\"name\",\"count\":1,\"bson_types\":[\"String\"],\"probability\":0.0,\"has_duplicates\":false,\"types\":{\"String\":{\"path\":\"name\",\"count\":1,\"bson_type\":\"String\",\"probability\":1.0,\"values\":[\"Chashu\"],\"has_duplicates\":false,\"schema\":null,\"unique\":null}}}}}");
  }

  #[test]
  fn it_updates_count() {
    let mut schema_parser = SchemaParser::new();
    assert_eq!(schema_parser.count, 0);
    let json_str = r#"{"name": "Chashu", "type": "Cat"}"#;
    schema_parser.write(&json_str).unwrap();
    assert_eq!(schema_parser.count, 1);
  }

  #[test]
  fn it_updates_fields() {
    let mut schema_parser = SchemaParser::new();
    let json_str = r#"{"name": "Chashu", "type": "Cat"}"#;
    schema_parser.write(&json_str).unwrap();
    let name = Bson::String("Nori".to_owned());
    schema_parser.update_field("name", &name);
    let vec = vec![
      ValueType::Str("Chashu".to_owned()),
      ValueType::Str("Nori".to_owned()),
    ];
    let field = schema_parser.fields.get("name");
    if let Some(field) = field {
      let field_type = field.types.get("String");
      if let Some(field_type) = field_type {
        assert_eq!(field_type.values, vec);
      }
    }
  }

  #[bench]
  fn bench_it_updates_fields(bench: &mut Bencher) {
    let mut schema_parser = SchemaParser::new();
    let json_str = r#"{"name": "Nori", "type": "Cat"}"#;
    schema_parser.write(&json_str).unwrap();
    let name = Bson::String("Chashu".to_owned());

    bench.iter(|| schema_parser.update_field("name", &name));
  }

  #[test]
  fn it_generates_fields() {
    let mut schema_parser = SchemaParser::new();
    let doc = doc! {
      "name": "Rey",
      "type": "Dog"
    };
    schema_parser.generate_field(doc, &None);
    assert_eq!(schema_parser.fields.len(), 2);
    if let Some(f) = schema_parser.fields.get("name") {
      if let Some(t) = f.types.get("String") {
        assert_eq!(t.values[0], ValueType::Str("Rey".to_string()));
      }
    }
    if let Some(f) = schema_parser.fields.get("type") {
      if let Some(t) = f.types.get("String") {
        assert_eq!(t.values[0], ValueType::Str("Dog".to_string()));
      }
    }
  }

  #[bench]
  fn bench_it_generates_fields_no_path(bench: &mut Bencher) {
    let mut schema_parser = SchemaParser::new();

    bench.iter(|| {
      let doc = doc! {
        "name": "Rey",
        "type": "Dog"
      };
      let n = test::black_box(doc);
      schema_parser.generate_field(n, &None)
    });
  }

  #[bench]
  fn bench_it_generates_fields_with_path(bench: &mut Bencher) {
    let mut schema_parser = SchemaParser::new();

    bench.iter(|| {
      let doc = doc! {
        "name": "Rey",
        "type": "Dog"
      };
      let n = test::black_box(doc);
      schema_parser.generate_field(n, &Some("treats".to_owned()))
    });
  }
}
