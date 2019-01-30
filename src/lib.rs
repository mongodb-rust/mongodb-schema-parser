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
//!   let result = schema_parser.to_json();
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

use std::mem;
use std::string::String;

mod field;
use crate::field::Field;

mod field_type;
use crate::field_type::FieldType;

mod value_type;
use crate::value_type::ValueType;

#[wasm_bindgen]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaParser {
  count: i64,
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
    let count = &self.count + 1;
    mem::replace(&mut self.count, count);
    self.generate_field(doc, &None);
    Ok(())
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
    // need to set count here as well
    // maybe store the names in a hash map so then it's easier to look up the key
    for field in &mut self.fields {
      if field.name == key {
        let mut has_duplicates = false;
        for field_type in &mut field.types {
          field_type.update_count();
          // this will also update for duplicate fields
          field_type.update_value(&value);
          field_type.set_unique();
          has_duplicates = field_type.get_duplicates();
          field_type.set_duplicates(has_duplicates);
        }
        field.set_duplicates(has_duplicates);
      }
    }
  }

  fn update_or_create_field(
    &mut self,
    key: String,
    value: &Bson,
    path: String,
    count: usize,
  ) {
    if self.does_field_name_exist(&key) {
      self.update_field(key, &value);
    } else {
      // if name doesn't exist, proceed by this path and create a new field
      let mut field = Field::new(key, &path, count);
      let field_type = FieldType::new(&path).add_to_type(&value);
      field.add_to_types(field_type.to_owned());
      self.add_to_fields(field);
    }
  }

  #[inline]
  fn generate_field(&mut self, doc: Document, path: &Option<String>) {
    let count = 0;

    for (key, value) in doc {
      // check if we already have a field for this key;
      // this check should also be checking for uniqueness
      // if name exist, call self.update_field -- should iterate over itself and call update field
      let current_path = Field::get_path(key.clone(), path);
      match &value {
        Bson::Document(subdoc) => {
          self.generate_field(subdoc.to_owned(), &Some(current_path));
        }
        _ => self.update_or_create_field(key, &value, current_path, count),
      };
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
    assert_eq!(schema_parser.fields.len(), 2)
  }

  #[bench]
  fn bench_it_writes(bench: &mut Bencher) {
    let mut schema_parser = SchemaParser::new();
    let json_str = r#"{"name": "Nori", "type": "Cat"}"#;
    bench.iter(|| schema_parser.write(&json_str));
  }

  #[test]
  fn it_formats_to_json() {
    let mut schema_parser = SchemaParser::new();
    let json_str = r#"{"name": "Chashu", "type": "Cat"}"#;
    schema_parser.write(&json_str).unwrap();
    let output = schema_parser.to_json().unwrap();
    assert_eq!(output, "{\"count\":1,\"fields\":[{\"name\":\"name\",\"path\":\"name\",\"count\":0,\"field_type\":null,\"probability\":null,\"has_duplicates\":false,\"types\":[{\"name\":\"String\",\"path\":\"name\",\"count\":0,\"bsonType\":\"String\",\"probability\":null,\"values\":[{\"Str\":\"Chashu\"}],\"has_duplicates\":false,\"unique\":null}]},{\"name\":\"type\",\"path\":\"type\",\"count\":0,\"field_type\":null,\"probability\":null,\"has_duplicates\":false,\"types\":[{\"name\":\"String\",\"path\":\"type\",\"count\":0,\"bsonType\":\"String\",\"probability\":null,\"values\":[{\"Str\":\"Cat\"}],\"has_duplicates\":false,\"unique\":null}]}]}");
  }

  #[test]
  fn it_adds_to_fields() {
    let mut schema_parser = SchemaParser::new();
    assert_eq!(schema_parser.fields.len(), 0);

    let name = "Nori";
    let path = "Nori.cat";
    let count = 1;
    let field = Field::new(&name, &path, count);

    schema_parser.add_to_fields(field);
    assert_eq!(schema_parser.fields.len(), 1);
  }

  #[bench]
  fn bench_it_adds_to_fields(bench: &mut Bencher) {
    let mut schema_parser = SchemaParser::new();
    let name = "Nori";
    let path = "Nori.cat";
    let count = 1;

    bench.iter(|| {
      let field = Field::new(&name, &path, count);
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
