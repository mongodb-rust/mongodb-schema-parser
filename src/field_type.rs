#![allow(clippy::option_map_unit_fn)]
use super::{Bson, SchemaParser, ValueType, HashMap, console};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FieldType {
  pub path: String,
  pub count: usize,
  pub bson_type: String,
  pub name: String,
  pub probability: f32,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub values: Vec<ValueType>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub lengths: Vec<usize>,
  pub has_duplicates: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(flatten)]
  pub schema: Option<SchemaParser>,
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub types: HashMap<String, FieldType>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub unique: Option<usize>,
}

pub static JAVASCRIPT_CODE_WITH_SCOPE: &str = "JavaScriptCodeWithScope";
pub static JAVASCRIPT_CODE: &str = "JavaScriptCode";
pub static FLOATING_POINT: &str = "Double";
pub static UTCDATE_TIME: &str = "UtcDatetime";
pub static DECIMAL_128: &str = "Decimal128";
pub static TIMESTAMP: &str = "Timestamp";
pub static BINARY: &str = "BinData";
pub static REGEXP: &str = "Regex";
pub static DOCUMENT: &str = "Document";
pub static OBJECTID: &str = "ObjectId";
pub static BOOLEAN: &str = "Boolean";
pub static SYMBOL: &str = "Symbol";
pub static STRING: &str = "String";
pub static ARRAY: &str = "Array";
pub static I32: &str = "Int32";
pub static I64: &str = "Long";
pub static NULL: &str = "Null";
pub static DB_POINTER: &str = "DbPointer";
pub static MAX_KEY: &str = "MaxKey";
pub static MIN_KEY: &str = "MinKey";
pub static UNDEFINED: &str = "Undefined";

impl FieldType {
  pub fn new<T, U>(path: T, bson_type: U) -> Self
  where
    T: Into<String>,
    U: Into<String> + Copy,
   {
    FieldType {
      path: path.into(),
      bson_type: bson_type.into(),
      count: 1,
      probability: 0.0,
      // name is the same as path, as there are several modules upstream that
      // look specifically at name field
      name: bson_type.into(),
      values: Vec::new(),
      has_duplicates: false,
      lengths: Vec::new(),
      schema: None,
      types: HashMap::new(),
      unique: None,
    }
  }

  pub fn add_to_type(&mut self, value: &Bson, parent_count: usize) {
    let bson_value = value.clone();
    self.set_probability(parent_count);

    match value {
      Bson::Array(arr) => {
        // push items into a types array for nested documents. if current item
        // type is a Document, create another schema parser;
        for val in arr.iter() {
          let current_type = Self::get_type(val);

          if self.types.contains_key(&current_type) {
            // console::log_2(&"current type contains key".into(), &current_type.clone().into());
            // console::log_2(&"current self field_type contains key".into(), &self.path.clone().into());
            self.types.get_mut(&current_type).unwrap().add_to_type(&val, self.count);
          } else {
            let mut field_type = FieldType::new(&self.path, &current_type);
            field_type.add_to_type(&val, self.count); // this is recursive
            self.types.insert(current_type, field_type.clone());
          }
          self.lengths.push(arr.len());
          Self::get_value(&val).map(|v| self.values.push(v));
        }
      }
      Bson::Document(subdoc) => {
        match self.schema.as_mut() {
          Some(doc) => {
            doc.generate_field(subdoc.to_owned(), Some(self.path.clone()), Some(self.count));
          },
          None => {
            let mut schema_parser = SchemaParser::new();
            schema_parser.generate_field(
              subdoc.to_owned(),
              Some(self.path.clone()),
              Some(self.count),
            );
            self.set_schema(schema_parser);
          }
        }
      }
      _ => {
        Self::get_value(&bson_value).map(|v| self.values.push(v));
      }
    }
  }

  pub fn update_type(&mut self, value: &Bson) {
    if self.bson_type == "Document" {
      match &mut self.schema {
        Some(schema_parser) => match &value {
          Bson::Document(subdoc) => schema_parser.generate_field(
            subdoc.to_owned(),
            Some(self.path.clone()),
            Some(self.count),
          ),
          _ => {
            unimplemented!()
          }
        },
        None => {
          unimplemented!()
        }
      }
    }

    self.update_count();
    self.update_value(&value);
  }

  fn update_value(&mut self, value: &Bson) {
    match value {
      Bson::Array(arr) => {
        for val in arr.iter() {
          let current_type = Self::get_type(val);

          if self.types.contains_key(&current_type) {
            self.types.get_mut(&current_type).unwrap().add_to_type(&val, self.count);
          } else {
            let mut field_type = FieldType::new(&self.path, &current_type);
            field_type.add_to_type(&val, self.count);
            self.types.insert(current_type, field_type.to_owned());
          }
          self.lengths.push(arr.len());
          Self::get_value(&val).map(|v| self.values.push(v));
        }
      }
      _ => {
        Self::get_value(&value).map(|v| self.values.push(v));
      }
    }
  }

  pub fn get_value(value: &Bson) -> Option<ValueType> {
    match value {
      Bson::JavaScriptCode(val)
      | Bson::Symbol(val) => Some(ValueType::Str(val.to_string())),
      Bson::JavaScriptCodeWithScope(val) => Some(ValueType::Str(val.to_string())),
      Bson::Int32(num) => Some(ValueType::I32(*num)),
      Bson::Int64(num) => Some(ValueType::I64(*num)),
      Bson::Timestamp(num) => Some(ValueType::I32(num.time as i32)),
      Bson::Double(num) => Some(ValueType::FloatingPoint(*num)),
      Bson::DateTime(date) => Some(ValueType::Str(date.clone().to_string())),
      Bson::Decimal128(d128) => Some(ValueType::Decimal128(d128.to_string())),
      Bson::Boolean(boolean) => Some(ValueType::Boolean(*boolean)),
      Bson::RegularExpression(val) => Some(ValueType::Str(val.to_string())),
      Bson::String(string) => Some(ValueType::Str(string.to_string())),
      Bson::Binary(vec) => Some(ValueType::Binary(vec.bytes.clone())),
      Bson::ObjectId(id) => Some(ValueType::Str(id.to_string())),
      Bson::Null => Some(ValueType::Null("Null".to_string())),
      // Array and Document get handeled separately
      _ => None,
    }
  }

  pub fn finalise_type(&mut self, parent_count: usize) {
    if !self.types.is_empty() {
      for (key, field) in self.types.iter_mut() {
        if key == "Document" {
          // TODO: get option without unwrap
          let schema = &mut field.schema;
          if let Some(schema) = schema {
            schema.finalise_schema();
          }
        }
        field.finalise_type(self.count);
      }
    }
    self.set_probability(parent_count);
    self.set_unique();
    self.set_duplicates();
  }

  pub fn get_type(value: &Bson) -> String {
    match value {
      Bson::JavaScriptCodeWithScope(_) => {
        JAVASCRIPT_CODE_WITH_SCOPE.to_string()
      }
      Bson::JavaScriptCode(_) => JAVASCRIPT_CODE.to_string(),
      Bson::Double(_) => FLOATING_POINT.to_string(),
      Bson::DateTime(_) => UTCDATE_TIME.to_string(),
      Bson::Decimal128(_) => DECIMAL_128.to_string(),
      Bson::Timestamp(_) => TIMESTAMP.to_string(),
      Bson::Binary(_) => BINARY.to_string(),
      Bson::RegularExpression(_) => REGEXP.to_string(),
      Bson::Document(_) => DOCUMENT.to_string(),
      Bson::ObjectId(_) => OBJECTID.to_string(),
      Bson::Boolean(_) => BOOLEAN.to_string(),
      Bson::Symbol(_) => SYMBOL.to_string(),
      Bson::String(_) => STRING.to_string(),
      Bson::Array(_) => ARRAY.to_string(),
      Bson::Int32(_) => I32.to_string(),
      Bson::Int64(_) => I64.to_string(),
      Bson::Null => NULL.to_string(),
      Bson::DbPointer(_) => DB_POINTER.to_string(),
      Bson::MaxKey => MAX_KEY.to_string(),
      Bson::MinKey => MIN_KEY.to_string(),
      Bson::Undefined => UNDEFINED.to_string(),
    }
  }

  fn get_duplicates(&mut self) -> bool {
    let unique = self.get_unique();
    let total_values = self.values.len();
    (total_values - unique) != 0
  }

  fn get_unique(&mut self) -> usize {
    let mut vec = self.values.clone();
    vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
    vec.dedup();
    vec.len()
  }

  pub fn set_duplicates(&mut self) {
    // if any nested types have duplicates, current field_type will then also
    // have duplicates.
    if !self.types.is_empty() {
      for (_key, field) in self.types.iter() {
        if field.has_duplicates {
          self.has_duplicates = field.has_duplicates
        }
      }
    }
    let duplicates = self.get_duplicates();
    self.has_duplicates = duplicates
  }

  fn set_schema(&mut self, schema: SchemaParser) {
    self.schema = Some(schema)
  }

  fn set_unique(&mut self) {
    self.unique = Some(self.get_unique())
  }

  fn set_probability(&mut self, parent_count: usize) {
    self.probability = self.count as f32 / parent_count as f32
  }

  fn update_count(&mut self) {
    self.count += 1
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  // use crate::test::Bencher;

  #[test]
  fn it_creates_new() {
    let address = "address";
    let field_type = FieldType::new(address, "Oranienstr. 123");
    assert_eq!(field_type.path, address);
  }

  // #[bench]
  // fn bench_it_creates_new(bench: &mut Bencher) {
  //   bench.iter(|| {
  //     FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()))
  //   });
  // }

  #[test]
  fn it_adds_to_type() {}

  #[test]
  fn it_gets_value_i32() {
    let bson_value = Bson::Int32(1234);
    let value = FieldType::get_value(&bson_value);
    assert_eq!(value, Some(ValueType::I32(1234)));
  }

  #[test]
  fn it_gets_value_i64() {
    let bson_value = Bson::Int64(1234);
    let value = FieldType::get_value(&bson_value);
    assert_eq!(value, Some(ValueType::I64(1234)));
  }

  #[test]
  fn it_gets_value_floating_point() {
    let bson_value = Bson::Double(1.2);
    let value = FieldType::get_value(&bson_value);
    assert_eq!(value, Some(ValueType::FloatingPoint(1.2)));
  }

  #[test]
  fn it_gets_value_boolean() {
    let bson_value = Bson::Boolean(true);
    let value = FieldType::get_value(&bson_value);
    assert_eq!(value, Some(ValueType::Boolean(true)));
  }

  #[test]
  fn it_gets_value_string() {
    let bson_value = Bson::String("cats".to_string());
    let value = FieldType::get_value(&bson_value);
    assert_eq!(value, Some(ValueType::Str("cats".to_string())));
  }

  // #[bench]
  // fn bench_it_gets_value(bench: &mut Bencher) {
  //   let bson_value = Bson::String("cats".to_string());
  //   bench.iter(|| FieldType::get_value(&bson_value));
  // }

  #[test]
  fn it_gets_type() {}

  #[allow(clippy::float_cmp)]
  #[test]
  fn it_sets_probability() {
    let mut field_type =
      FieldType::new("address", "Oranienstr. 123");
    field_type.set_probability(10);
    assert_eq!(field_type.probability, 0.1);
  }

  #[test]
  fn it_gets_unique() {
    let mut field_type =
      FieldType::new("address", "Oranienstr. 123");
    field_type.values.push(ValueType::Str("Berlin".to_string()));
    field_type
      .values
      .push(ValueType::Str("Hamburg".to_string()));
    let unique = field_type.get_unique();
    assert_eq!(unique, 2);
  }

  // #[bench]
  // fn bench_it_gets_unique(bench: &mut Bencher) {
  //   let mut field_type =
  //     FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
  //   field_type.values.push(ValueType::Str("Berlin".to_string()));
  //   field_type
  //     .values
  //     .push(ValueType::Str("Hamburg".to_string()));
  //   bench.iter(|| field_type.get_unique());
  // }

  #[test]
  fn it_sets_unique() {
    let mut field_type =
      FieldType::new("address", "Oranienstr. 123");
    field_type.values.push(ValueType::Str("Berlin".to_string()));
    field_type
      .values
      .push(ValueType::Str("Hamburg".to_string()));
    field_type.set_unique();
    assert_eq!(field_type.unique, Some(2));
  }

  // #[bench]
  // fn bench_it_sets_unique(bench: &mut Bencher) {
  //   let mut field_type =
  //     FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
  //   field_type.values.push(ValueType::Str("Berlin".to_string()));
  //   field_type
  //     .values
  //     .push(ValueType::Str("Hamburg".to_string()));
  //   bench.iter(|| field_type.set_unique());
  // }

  #[test]
  fn it_gets_duplicates_when_none() {
    let mut field_type =
      FieldType::new("address", "Oranienstr. 123");
    field_type.values.push(ValueType::Str("Berlin".to_string()));
    field_type
      .values
      .push(ValueType::Str("Hamburg".to_string()));
    let has_duplicates = field_type.get_duplicates();
    assert_eq!(has_duplicates, false)
  }

  #[test]
  fn it_gets_duplicates_when_some() {
    let mut field_type =
      FieldType::new("address", "Oranienstr. 123");
    field_type.values.push(ValueType::Str("Berlin".to_string()));
    field_type.values.push(ValueType::Str("Berlin".to_string()));
    let has_duplicates = field_type.get_duplicates();
    assert_eq!(has_duplicates, true)
  }

  // #[bench]
  // fn bench_it_gets_duplicates(bench: &mut Bencher) {
  //   let mut field_type =
  //     FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
  //   field_type.values.push(ValueType::Str("Berlin".to_string()));
  //   field_type
  //     .values
  //     .push(ValueType::Str("Hamburg".to_string()));
  //   bench.iter(|| field_type.get_duplicates());
  // }

  #[test]
  fn it_sets_duplicates() {
    let mut field_type =
      FieldType::new("address", "Oranienstr. 123");
    field_type.values.push(ValueType::Str("Berlin".to_string()));
    field_type.values.push(ValueType::Str("Berlin".to_string()));
    field_type.set_duplicates();
    assert_eq!(field_type.has_duplicates, true)
  }

  // #[bench]
  // fn bench_it_sets_duplicates(bench: &mut Bencher) {
  //   let mut field_type =
  //     FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
  //   bench.iter(|| field_type.set_duplicates());
  // }

  #[test]
  fn it_updates_count() {
    let mut field_type =
      FieldType::new("address", "Oranienstr. 123");
    field_type.update_count();
    assert_eq!(field_type.count, 2);
  }

  // #[bench]
  // fn bench_it_updates_count(bench: &mut Bencher) {
  //   let mut field_type =
  //     FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
  //   bench.iter(|| field_type.update_count());
  // }

  #[test]
  fn it_updates_value_some() {
    let bson_value = Bson::Int32(1234);
    let mut field_type =
      FieldType::new("address", "Oranienstr. 123");
    field_type.update_value(&bson_value);
    assert_eq!(field_type.values[0], ValueType::I32(1234));
  }

  // #[bench]
  // fn bench_it_updates_value_some(bench: &mut Bencher) {
  //   let bson_value = Bson::Int32(1234);
  //   let mut field_type =
  //     FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
  //   bench.iter(|| field_type.update_value(&bson_value));
  // }
}
