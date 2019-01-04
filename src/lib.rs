#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(feature = "nightly", deny(unsafe_code))]
#![allow(clippy::new_without_default_derive)]
//#![cfg_attr(test, deny(warnings))]

extern crate bson;
use bson::{Bson, Document};

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
use serde_json::Value;

use std::mem;
use std::string::String;

mod field;
use crate::field::Field;

mod field_type;
use crate::field_type::FieldType;

mod value_type;
use crate::value_type::ValueType;

// mod error;
// pub use error::{Error, ErrorKind, Result};

extern crate failure;
/// Custom Result type
pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaParser {
  count: i64,
  fields: Vec<Field>,
}

impl SchemaParser {
  pub fn new() -> Self {
    SchemaParser {
      count: 0,
      fields: Vec::new(),
    }
  }

  pub fn write(&mut self, json: &str) -> Result<()> {
    let val: Value = serde_json::from_str(json)?;
    let bson = Bson::from(val);
    // should do a match for NoneError
    let doc = bson.as_document().unwrap().to_owned();
    let count = &self.count + 1;
    mem::replace(&mut self.count, count);
    self.generate_field(doc, &None);
    Ok(())
  }

  pub fn to_json(&self) -> Result<String> {
    Ok(serde_json::to_string(&self)?)
  }

  fn add_to_fields(&mut self, field: Field) {
    self.fields.push(field)
  }

  // why do i have to explicitly return true instead of just returning field.name == key
  fn does_field_name_exist(&mut self, key: &str) -> bool {
    for field in &mut self.fields {
      if field.name == key {
        return true;
      }
    }
    false
  }

  fn update_field(&mut self, key: &str, value: &Bson) {
    // need to set count here as well
    // maybe store the names in a hash map so then it's easier to look up the key
    for field in &mut self.fields {
      if field.name == key {
        for field_type in &mut field.types {
          // update field type,
          field_type.update_count();
          field_type.update_value(&value);
        }
      }
    }
  }

  fn generate_field(&mut self, doc: Document, path: &Option<String>) {
    let count = 0;

    for (key, value) in doc {
      // check if we already have a field for this key;
      // this check should also be checking for uniqueness
      // 'inner:
      // if name exist, call self.update_field -- should iterate over itself and call update field
      if self.does_field_name_exist(&key) {
        self.update_field(&key, &value);
      } else {
        // if name doesn't exist, proceed by this path and create a new field
        let current_path = Field::get_path(key.clone(), path);
        let mut field = Field::new(&key, &current_path, count);

        match &value {
          Bson::Document(subdoc) => {
            self.generate_field(subdoc.to_owned(), &Some(current_path));
          }
          _ => {
            let field_type = FieldType::new(&current_path).add_to_type(&value);
            field.add_to_types(field_type.to_owned());
          }
        };
        self.add_to_fields(field);
      }
    }
  }
}
