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
  count: usize,
  fields: Vec<Field>,
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
  pub fn wasm_to_json(&mut self) -> Result<String, JsValue> {
    match self.to_json() {
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
      fields: Vec::new(),
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
  pub fn update_count(&mut self) {
    self.count += 1
  }

  #[inline]
  fn add_to_fields(&mut self, field: Field) {
    self.fields.push(field)
  }

  // why do i have to explicitly return true instead of just returning field.name == key
  #[inline]
  fn does_field_name_exist(&mut self, key: &str) -> bool {
    for field in &mut self.fields {
      if field.name == key {
        return true;
      }
    }
    false
  }

  #[inline]
  fn update_field(&mut self, key: &str, value: &Bson) {
    for field in &mut self.fields {
      if field.name == key {
        let mut has_duplicates = false;
        field.update_count();
        if !field.does_field_type_exist(FieldType::get_type(&value)) {
          // field type doesn't exist in field.types, create a new field_type
          field.create_type(&value);
        } else {
          // update field_type based on bson_type
          for field_type in &mut field.types {
            if field_type.bsonType == FieldType::get_type(&value) {
              field_type.update_type(field.count, &value);
              has_duplicates = field_type.get_duplicates();
            }
          }
        }
        field.set_duplicates(has_duplicates);
        field.set_probability(self.count);
      }
    }
  }

  #[inline]
  fn update_or_create_field(&mut self, key: String, value: &Bson, path: &str) {
    // check if we already have a field for this key;
    // if name exist, call self.update_field, otherwise create new
    if self.does_field_name_exist(&key) {
      self.update_field(&key, &value);
    } else {
      let mut field = Field::new(key, &path);
      field.create_type(&value);
      self.add_to_fields(field);
    }
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
    assert_eq!(output, "{\"count\":1,\"fields\":[{\"name\":\"name\",\"path\":\"name\",\"count\":1,\"field_type\":null,\"probability\":0.0,\"has_duplicates\":false,\"types\":[{\"name\":\"String\",\"path\":\"name\",\"count\":1,\"bsonType\":\"String\",\"probability\":0.0,\"values\":[\"Chashu\"],\"has_duplicates\":false,\"unique\":null}]},{\"name\":\"type\",\"path\":\"type\",\"count\":1,\"field_type\":null,\"probability\":0.0,\"has_duplicates\":false,\"types\":[{\"name\":\"String\",\"path\":\"type\",\"count\":1,\"bsonType\":\"String\",\"probability\":0.0,\"values\":[\"Cat\"],\"has_duplicates\":false,\"unique\":null}]}]}");
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
  fn it_adds_to_fields() {
    let mut schema_parser = SchemaParser::new();
    assert_eq!(schema_parser.fields.len(), 0);

    let name = "Nori".to_string();
    let path = "Nori.cat";
    let field = Field::new(name, &path);

    schema_parser.add_to_fields(field);
    assert_eq!(schema_parser.fields.len(), 1);
  }

  #[bench]
  fn bench_it_adds_to_fields(bench: &mut Bencher) {
    let mut schema_parser = SchemaParser::new();
    let path = "Nori.cat";

    bench.iter(|| {
      let field = Field::new("Nori".to_string(), &path);
      let n = test::black_box(field);
      schema_parser.add_to_fields(n)
    });
  }

  #[test]
  fn it_checks_if_field_name_exists() {
    let mut schema_parser = SchemaParser::new();
    let json_str = r#"{"name": "Chashu", "type": "Cat"}"#;
    schema_parser.write(&json_str).unwrap();
    assert_eq!(schema_parser.does_field_name_exist("name"), true);
    assert_eq!(schema_parser.does_field_name_exist("colour"), false);
  }

  #[bench]
  fn bench_it_check_if_field_name_exists(bench: &mut Bencher) {
    let mut schema_parser = SchemaParser::new();
    let json_str = r#"{"name": "Chashu", "type": "Cat"}"#;
    schema_parser.write(&json_str).unwrap();

    bench.iter(|| schema_parser.does_field_name_exist("name"));
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
    assert_eq!(schema_parser.fields[0].types[0].values, vec);
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
    assert_eq!(schema_parser.fields[0].name, "name");
    assert_eq!(schema_parser.fields[1].name, "type");
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
