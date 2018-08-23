#![cfg_attr(feature = "nightly", deny(missing_docs))]
#![cfg_attr(feature = "nightly", feature(external_doc))]
#![cfg_attr(feature = "nightly", doc(include = "../README.md"))]
#![cfg_attr(feature = "nightly", deny(unsafe_code))]
#![cfg_attr(test, deny(warnings))]

#[macro_use]
extern crate failure;

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

mod error;

pub use error::{Error, ErrorKind, Result};

#[wasm_bindgen]
pub fn parses(schema: &str) { 
}
