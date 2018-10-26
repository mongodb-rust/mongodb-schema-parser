#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(feature = "nightly", deny(unsafe_code))]
//#![cfg_attr(test, deny(warnings))]

#[macro_use]
extern crate bson;
#[macro_use]
extern crate failure;

// #[macro_use]
// extern crate mongodb;
// use mongodb::{Client, ThreadedClient};
// use mongodb::db::ThreadedDatabase;
// use mongodb::coll::options::FindOptions;

// #[macro_use]
// extern crate bson;
// use bson::Bson;

use bson::{Bson, Document};

use std::string::String;
mod error;
pub use error::{Error, ErrorKind, Result};

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

pub fn parser() -> MongoDBSchema {
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

  mongodb_schema
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn can_generate_schema() {
    // let client = Client::connect("localhost", 27017).expect("Failed to initialize standalone client.");

    // let coll = client.db("crunchbase").collection("companies");
    // let limit_option: Option<i64> = Some(1);
    // let find_option = Bson::OrderedDocument {
    //   name: "AdventNet",
    // };
    // // create find_options passing in limit ?
    // // let find_options =
    // let mut cursor = coll.find(Some(find_option), None)
    //   .ok().expect("Failed to execute find.");

    // let item = cursor.next();

    // match item {
    //   Some(Ok(doc)) => match doc.get("name") {
    //     Some(&Bson::String(ref name)) => assert_eq!(name, "AdventNet"),
    //     _ => panic!("Expected name to be AdventNet"),
    //   },
    //   Some(Err(_)) => panic!("Cannot get next cursor from server"),
    //   None => panic!("Cannot obtain document from server"),
    // }

    let mongodb_schema = parser();
    let count: usize = 4;
    let field_name = String::from("_id");
    assert_eq!(&mongodb_schema.count, &count);
    assert_eq!(&mongodb_schema.fields[0].name, &field_name);
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
