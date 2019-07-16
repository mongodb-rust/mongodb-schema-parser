use super::SchemaParser;
use js_sys::{Object, Uint8Array};
use wasm_bindgen::prelude::*;
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
  /// var result = schemaParser.toObject()
  /// console.log(result) //
  /// ````
  #[wasm_bindgen(js_name = "toObject")]
  pub fn wasm_to_js_object(&mut self) -> Result<Object, JsValue> {
    self.flush();
    match self.to_js_object() {
      Err(e) => Err(JsValue::from_str(&format!("{}", e))),
      Ok(val) => Ok(val),
    }
  }
}
