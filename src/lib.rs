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
//!     schema_parser.write_json(&json).unwrap();
//!   }
//!   let result = schema_parser.flush();
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
//!         schemaParser.writeJson(json[i])
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
#![allow(unused_imports)]
#![allow(clippy::new_without_default)]
// #![feature(test)]

extern crate failure;
// extern crate test;

extern crate bson;
use bson::{bson, decode_document, doc, Bson, Document};

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
use serde_json::Value;

use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
// add to use console.log to send debugs to js land
use web_sys::console;

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
  count: usize,
  fields: HashMap<String, Field>,
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
  /// * `json` - A json-like string slice. i.e `{ "name": "Nori", "type": "Cat"}`
  ///
  /// # Examples
  /// ```
  /// use mongodb_schema_parser::SchemaParser;
  /// let mut schema_parser = SchemaParser::new();
  /// let json = r#"{ "name": "Chashu", "type": "Cat" }"#;
  /// schema_parser.write_json(&json);
  /// ```
  #[inline]
  pub fn write_json(&mut self, json: &str) -> Result<(), failure::Error> {
    let val: Value = serde_json::from_str(json)?;
    let bson = Bson::from(val);
    // should do a match for NoneError
    let doc = bson.as_document().unwrap().to_owned();
    self.update_count();
    self.generate_field(doc, None, None);
    Ok(())
  }

  /// Writes Bson documents to SchemaParser's fields vector.
  ///
  /// # Arguments
  /// * `doc` - A Bson Document.
  ///
  /// # Examples
  /// ```ignore
  /// use mongodb_schema_parser::SchemaParser;
  /// use js_sys::Uint8Array;
  /// use bson::{doc, bson};
  ///
  /// let mut schema_parser = SchemaParser::new();
  /// let uint8 = Uint8Array::new(&JsValue::from_str(r#"{ "name": "Chashu", "type": "Cat" }"#));
  /// schema_parser.write_raw(uint8);
  /// ```
  #[inline]
  pub fn write_raw(&mut self, uint8: Uint8Array) -> Result<(), failure::Error> {
    let mut decoded_vec = vec![0u8; uint8.length() as usize];
    // fill up a new u8 vec with bytes we get from js; decode_document needs a
    // byte stream that implements a reader and u8 slice does this.
    uint8.copy_to(&mut decoded_vec);
    let mut slice: &[u8] = &decoded_vec;
    let doc = decode_document(&mut slice)?.to_owned();
    console::log_1(&"can decode document".into());
    // write bson internally
    self.update_count();
    self.generate_field(doc, None, None);

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
  /// schema_parser.write_json(&json);
  /// let schema = schema_parser.flush();
  /// println!("{:?}", schema);
  /// ```
  pub fn flush(&mut self) -> SchemaParser {
    self.finalise_schema();
    self.to_owned()
  }

  /// Returns a serde_json string. This should be called after all values were
  /// written. This is also the result of the parsed documents.
  ///
  /// # Examples
  /// ```
  /// use mongodb_schema_parser::SchemaParser;
  /// let mut schema_parser = SchemaParser::new();
  /// let json = r#"{ "name": "Chashu", "type": "Cat" }"#;
  /// schema_parser.write_json(&json);
  /// schema_parser.flush();
  /// let schema = schema_parser.to_json().unwrap();
  /// println!("{}", schema);
  /// ```
  #[inline]
  pub fn to_json(&self) -> Result<String, failure::Error> {
    Ok(serde_json::to_string(&self)?)
  }

  #[inline]
  fn generate_field(
    &mut self,
    doc: Document,
    path: Option<String>,
    count: Option<&usize>,
  ) {
    if let Some(_count) = count {
      self.update_count();
    }
    for (key, value) in doc {
      let current_path = Field::get_path(key.to_owned(), path.to_owned());
      self.update_or_create_field(key.to_owned(), &value, &current_path)
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
      field.update_count();
      if !field.does_field_type_exist(&value) {
        // field type doesn't exist in field.types, create a new field_type
        field.create_type(&value);
      } else {
        let type_val = FieldType::get_type(&value);
        let field_type = field.types.get_mut(&type_val);
        if let Some(field_type) = field_type {
          field_type.update_type(&value);
        }
      }
    }
  }

  #[inline]
  fn finalise_schema(&mut self) {
    for field in self.fields.values_mut() {
      // If bson_types includes a Document, find that document and let its schema
      // field update its own missing fields.
      let field_type = field.types.get_mut(&"Document".to_string());
      if let Some(field_type) = field_type {
        let schema = &mut field_type.schema;
        if let Some(schema) = schema {
          return schema.finalise_schema();
        }
      }

      // create new field_types as Null for missing fields
      let missing = self.count - field.count;
      if missing > 0 {
        field.update_for_missing(missing);
      }

      // check for duplicates, unique values, set probability
      field.finalise_field(self.count);
    }
  }

  #[inline]
  fn update_count(&mut self) {
    self.count += 1
  }
}

// Need to wrap schema parser impl for wasm suppport.
// Here we are wrapping the exported to JS land methods and mathing on Result to
// turn the error message to JsValue.
#[wasm_bindgen]
impl SchemaParser {
  /// Wrapper method for `SchemaParser::new()` to be used in JavaScript.
  /// `wasm_bindgen(js_name = "new")`
  ///
  /// ```js, ignore
  /// import { SchemaParser } from "mongodb-schema-parser";
  ///
  /// var schemaParser = new SchemaParser()
  /// ````
  #[wasm_bindgen(constructor)]
  #[wasm_bindgen(js_name = "new")]
  pub fn wasm_new() -> Self {
    Self::new()
  }

  /// Wrapper method for `schema_parser.write_json()` to be used in JavaScript.
  /// `wasm_bindgen(js_name = "writeJson")`
  ///
  ///
  /// ```js, ignore
  /// import { SchemaParser } from "mongodb-schema-parser"
  ///
  /// var schemaParser = new SchemaParser()
  /// var json = "{"name": "Nori", "type": "Cat"}"
  /// schemaParser.writeJson(json)
  /// ````
  #[wasm_bindgen(js_name = "writeJson")]
  pub fn wasm_write_json(&mut self, json: &str) -> Result<(), JsValue> {
    match self.write_json(json) {
      Err(e) => Err(JsValue::from_str(&format!("{}", e))),
      _ => Ok(()),
    }
  }

  #[wasm_bindgen(js_name = "writeRaw")]
  pub fn wasm_write_raw(&mut self, uint8: Uint8Array) -> Result<(), JsValue> {
    match self.write_raw(uint8) {
      Err(e) => Err(JsValue::from_str(&format!("{}", e))),
      _ => Ok(()),
    }
  }

  /// Wrapper method for `schema_parser.to_json()` to be used in JavaScript.
  /// `wasm_bindgen(js_name = "toJson")`
  ///
  /// ```js, ignore
  /// import { SchemaParser } from "mongodb-schema-parser"
  ///
  /// var schemaParser = new SchemaParser()
  /// var json = "{"name": "Nori", "type": "Cat"}"
  /// schemaParser.writeJson(json)
  /// // get the result as a json string
  /// var result = schemaParser.toJson()
  /// console.log(result) //
  /// ````
  #[wasm_bindgen(js_name = "toJson")]
  pub fn wasm_to_json(&mut self) -> Result<String, JsValue> {
    self.flush();
    match self.to_json() {
      Err(e) => Err(JsValue::from_str(&format!("{}", e))),
      Ok(val) => Ok(val),
    }
  }
}

#[cfg(test)]
mod tests {
  // use self::test::Bencher;
  use super::*;

  #[test]
  fn it_creates_new() {
    let schema_parser = SchemaParser::new();
    assert_eq!(schema_parser.count, 0);
    assert_eq!(schema_parser.fields.len(), 0);
  }

  // #[bench]
  // fn bench_it_creates_new(bench: &mut Bencher) {
  //   bench.iter(|| SchemaParser::new);
  // }

  #[test]
  fn it_writes_json() {
    let mut schema_parser = SchemaParser::new();
    let json_str = r#"{"name": "Nori", "type": "Cat"}"#;
    schema_parser.write_json(&json_str).unwrap();
    assert_eq!(schema_parser.count, 1);
    assert_eq!(schema_parser.fields.len(), 2);
  }

  // #[bench]
  // fn bench_it_writes_json(bench: &mut Bencher) {
  //   let mut schema_parser = SchemaParser::new();
  //   let json_str = r#"{"name": "Nori", "type": "Cat"}"#;
  //   bench.iter(|| schema_parser.write_json(&json_str));
  // }

  // #[test]
  // fn it_writes_bson() {
  //   let mut schema_parser = SchemaParser::new();
  //   // let bson_str = bson!({"name": "Nori", "type": "Cat"});
  //   schema_parser.write_raw(&bson_str).unwrap();
  //   println!("{:?}", schema_parser.flush());
  //   assert_eq!(schema_parser.count, 1);
  //   assert_eq!(schema_parser.fields.len(), 2);
  // }

  // #[bench]
  // fn bench_it_creates_write_json(bench: &mut Bencher) {
  //   let mut schema_parser = SchemaParser::new();
  //   let json_str = r#"{"name": "Nori", "type": "Cat"}"#;
  //   bench.iter(|| schema_parser.write_json(&json_str));
  // }

  #[test]
  fn it_flushes() {
    let mut schema_parser = SchemaParser::new();
    let json_str = r#"{"name": "Nori", "type": "Cat"}"#;
    schema_parser.write_json(&json_str).unwrap();
    let output = schema_parser.flush();
    println!("{:?}", output);
    assert_eq!(output.count, 1);
    assert_eq!(output.fields.len(), 2);
  }

  #[test]
  fn it_adjusts_missing() {
    let mut schema_parser = SchemaParser::new();
    let json_str1 = r#"{"name": "Nori", "type": "Cat"}"#;
    let json_str2 = r#"{"name": "Rey"}"#;
    let json_str3 = r#"{"name": "Chashu"}"#;
    schema_parser.write_json(&json_str1).unwrap();
    schema_parser.write_json(&json_str2).unwrap();
    schema_parser.write_json(&json_str3).unwrap();
    let mut output = schema_parser.flush();
    let type_field = output.fields.get_mut("type");
    if let Some(type_field) = type_field {
      assert_eq!(type_field.count, 3);
      assert!(type_field.bson_types.contains(&"Null".to_string()));

      let null_field_type = type_field.types.get_mut("Null");
      if let Some(null_field_type) = null_field_type {
        assert_eq!(null_field_type.count, 2)
      }
    }
  }

  #[test]
  fn it_adjusts_missing_with_nested_document() {
    let mut schema_parser = SchemaParser::new();
    let json_str1 = r#"{"name": "Nori", "type": {"breed": "Norwegian Forest", "type": "cat"}}"#;
    let json_str2 = r#"{"name": "Rey", "type": {"breed": "Viszla"}}"#;
    schema_parser.write_json(&json_str1).unwrap();
    schema_parser.write_json(&json_str2).unwrap();
    let output = schema_parser.flush();
    let type_field = output.fields.get("type");
    if let Some(type_field) = type_field {
      let doc = type_field.types.get("Document");
      if let Some(doc) = doc {
        assert_eq!(doc.count, 2);

        let schema = &doc.schema;
        if let Some(schema) = schema {
          assert_eq!(schema.count, 2);

          let type_schema_field = schema.fields.get("type");
          if let Some(type_schema_field) = type_schema_field {
            assert_eq!(type_schema_field.count, 2);
            assert!(type_schema_field.bson_types.contains(&"Null".to_string()));

            let null_type_schema_field = type_schema_field.types.get("Null");
            if let Some(null_type_schema_field) = null_type_schema_field {
              assert_eq!(null_type_schema_field.count, 1)
            }
          }
        }
      }
    }
  }

  #[test]
  fn it_updates_count() {
    let mut schema_parser = SchemaParser::new();
    assert_eq!(schema_parser.count, 0);
    let json_str = r#"{"name": "Chashu", "type": "Cat"}"#;
    schema_parser.write_json(&json_str).unwrap();
    assert_eq!(schema_parser.count, 1);
  }

  #[test]
  fn it_updates_fields() {
    let mut schema_parser = SchemaParser::new();
    let json_str = r#"{"name": "Chashu", "type": "Cat"}"#;
    schema_parser.write_json(&json_str).unwrap();
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

  // #[bench]
  // fn bench_it_updates_fields(bench: &mut Bencher) {
  //   let mut schema_parser = SchemaParser::new();
  //   let json_str = r#"{"name": "Nori", "type": "Cat"}"#;
  //   schema_parser.write_json(&json_str).unwrap();
  //   let name = Bson::String("Chashu".to_owned());

  //   bench.iter(|| schema_parser.update_field("name", &name));
  // }

  #[test]
  fn it_generates_fields() {
    let mut schema_parser = SchemaParser::new();
    let doc = doc! {
      "name": "Rey",
      "type": "Dog"
    };
    schema_parser.generate_field(doc, None, None);
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

  // #[bench]
  // fn bench_it_generates_fields_no_path(bench: &mut Bencher) {
  //   let mut schema_parser = SchemaParser::new();

  //   bench.iter(|| {
  //     let doc = doc! {
  //       "name": "Rey",
  //       "type": "Dog"
  //     };
  //     let n = test::black_box(doc);
  //     schema_parser.generate_field(n, None, None)
  //   });
  // }

  // #[bench]
  // fn bench_it_generates_fields_with_path(bench: &mut Bencher) {
  //   let mut schema_parser = SchemaParser::new();

  //   bench.iter(|| {
  //     let doc = doc! {
  //       "name": "Rey",
  //       "type": "Dog"
  //     };
  //     let n = test::black_box(doc);
  //     schema_parser.generate_field(n, Some("treats".to_owned()), None)
  //   });
  // }

  #[test]
  fn it_combines_arrays_for_same_field_into_same_types_vector() {
    let mut schema_parser = SchemaParser::new();
    let vec_json1 = r#"{"animals": ["cat", "dog"]}"#;
    let vec_json2 = r#"{"animals": ["wallaby", "bird"]}"#;
    schema_parser.write_json(vec_json1).unwrap();
    schema_parser.write_json(vec_json2).unwrap();
    assert_eq!(schema_parser.fields.len(), 1);
    let field = schema_parser.fields.get("animals");
    if let Some(field) = field {
      assert_eq!(field.types.len(), 1);
      let field_type = field.types.get("Array");
      if let Some(field_type) = field_type {
        assert_eq!(field_type.values.len(), 4);
      }
    }
  }

  #[test]
  fn it_creates_different_field_types() {
    let mut schema_parser = SchemaParser::new();
    let number_json = r#"{"phone_number": 491234568789}"#;
    let string_json = r#"{"phone_number": "+441234456789"}"#;
    schema_parser.write_json(number_json).unwrap();
    schema_parser.write_json(string_json).unwrap();
    let field = schema_parser.fields.get("phone_number");
    if let Some(field) = field {
      assert_eq!(field.count, 2);
      assert_eq!(field.bson_types.len(), 2);
      assert_eq!(field.types.len(), 2);
    }
  }

  #[test]
  fn it_creates_field_type_for_null() {
    let mut schema_parser = SchemaParser::new();
    let null_json = r#"{"phone_number": null}"#;
    schema_parser.write_json(null_json).unwrap();
    let field = schema_parser.fields.get("phone_number");
    if let Some(field) = field {
      assert_eq!(field.bson_types[0], "Null");
    }
  }
}
