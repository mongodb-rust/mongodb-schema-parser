#![allow(clippy::option_map_unit_fn)]
use super::{Bson, SchemaParser, ValueType};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct FieldType {
  pub path: String,
  pub count: usize,
  pub bson_type: String,
  pub probability: f32,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub values: Vec<ValueType>,
  pub has_duplicates: bool,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(flatten)]
  pub schema: Option<SchemaParser>,
  pub unique: Option<usize>,
}

impl FieldType {
  pub fn new(path: &str, value: &Bson) -> Self {
    FieldType {
      path: path.to_string(),
      bson_type: FieldType::get_type(&value),
      count: 1,
      probability: 0.0,
      values: Vec::new(),
      has_duplicates: false,
      // serde json should remove when null
      // on finalize method, should also destructure it somehow (everything from
      // this structure should come up one level)
      schema: None,
      unique: None,
    }
  }

  pub fn add_to_type(mut self, value: &Bson, parent_count: usize) -> Self {
    let bson_value = value.clone();
    self.set_probability(parent_count);

    match value {
      Bson::Array(arr) => {
        for val in arr.iter() {
          let bson_val = val.clone();
          match val {
            Bson::Document(subdoc) => {
              let schema = self.parse_document(subdoc.to_owned());
              self.bson_type = "Document".to_string();
              self.set_schema(schema);
            }
            _ => {
              Self::get_value(&bson_val).map(|v| self.values.push(v));
            }
          };
        }
        self
      }
      Bson::Document(subdoc) => {
        let schema = self.parse_document(subdoc.to_owned());
        self.set_schema(schema);
        self
      }
      _ => {
        Self::get_value(&bson_value).map(|v| self.values.push(v));
        self
      }
    }
  }

  pub fn update_type(&mut self, value: &Bson) {
    let bson_type = self.bson_type.clone();

    if &bson_type == "Document" {
      match &mut self.schema {
        Some(_) => match &value {
          Bson::Document(subdoc) => {
            let schema = self.parse_document(subdoc.to_owned());
            self.set_schema(schema)
          }
          _ => unimplemented!(),
        },
        None => unimplemented!(),
      }
    }

    self.update_count();
    self.update_value(&value);
  }

  fn update_value(&mut self, value: &Bson) {
    match value {
      Bson::Array(arr) => {
        for val in arr.iter() {
          let bson_val = val.clone();
          match val {
            Bson::Document(_) => self.update_type(&bson_val),
            Bson::Array(_) => self.update_value(&bson_val),
            _ => {
              Self::get_value(&bson_val).map(|v| self.values.push(v));
            }
          }
        }
      }
      _ => {
        Self::get_value(&value).map(|v| self.values.push(v));
      }
    }
  }

  fn parse_document(
    &mut self,
    subdoc: bson::ordered::OrderedDocument,
  ) -> SchemaParser {
    let mut schema_parser = SchemaParser::new();
    schema_parser.generate_field(
      subdoc,
      Some(self.path.clone()),
      Some(&self.count),
    );
    schema_parser
  }

  pub fn get_value(value: &Bson) -> Option<ValueType> {
    match value {
      Bson::RegExp(val, _)
      | Bson::JavaScriptCode(val)
      | Bson::JavaScriptCodeWithScope(val, _)
      | Bson::Symbol(val) => Some(ValueType::Str(val.to_string())),
      Bson::I64(num) | Bson::TimeStamp(num) => Some(ValueType::I64(*num)),
      Bson::FloatingPoint(num) => Some(ValueType::FloatingPoint(*num)),
      Bson::UtcDatetime(date) => Some(ValueType::Str(date.clone().to_string())),
      Bson::Decimal128(d128) => Some(ValueType::Decimal128(d128.to_string())),
      Bson::Boolean(boolean) => Some(ValueType::Boolean(*boolean)),
      Bson::String(string) => Some(ValueType::Str(string.to_string())),
      Bson::Binary(_, vec) => Some(ValueType::Binary(vec.clone())),
      Bson::ObjectId(id) => Some(ValueType::Str(id.to_string())),
      Bson::I32(num) => Some(ValueType::I32(*num)),
      Bson::Null => Some(ValueType::Null("Null".to_string())),
      // Array and Document get handeled separately
      _ => None,
    }
  }

  pub fn finalise_type(&mut self, parent_count: usize) {
    self.set_probability(parent_count);
    if self.bson_type != "Document" {
      self.set_unique();
      self.set_duplicates();
    }
  }

  pub fn get_type(value: &Bson) -> String {
    match value {
      Bson::JavaScriptCodeWithScope(_, _) => {
        "JavaScriptCodeWithScope".to_string()
      }
      Bson::JavaScriptCode(_) => "JavaScriptCode".to_string(),
      Bson::FloatingPoint(_) => "Double".to_string(),
      Bson::UtcDatetime(_) => "UtcDatetime".to_string(),
      Bson::Decimal128(_) => "Decimal128".to_string(),
      Bson::TimeStamp(_) => "Timestamp".to_string(),
      Bson::Binary(_, _) => "BinData".to_string(),
      Bson::RegExp(_, _) => "Regex".to_string(),
      Bson::Document(_) => "Document".to_string(),
      Bson::ObjectId(_) => "ObjectId".to_string(),
      Bson::Boolean(_) => "Boolean".to_string(),
      Bson::Symbol(_) => "Symbol".to_string(),
      Bson::String(_) => "String".to_string(),
      Bson::Array(_) => "Array".to_string(),
      Bson::I32(_) => "Int".to_string(),
      Bson::I64(_) => "Long".to_string(),
      Bson::Null => "Null".to_string(),
    }
  }

  pub fn set_duplicates(&mut self) {
    let duplicates = self.get_duplicates();
    self.has_duplicates = duplicates
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
    let bson_string = Bson::String("Oranienstr. 123".to_string());
    let field_type = FieldType::new(address, &bson_string);
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
    let bson_value = Bson::I32(1234);
    let value = FieldType::get_value(&bson_value);
    assert_eq!(value, Some(ValueType::I32(1234)));
  }

  #[test]
  fn it_gets_value_i64() {
    let bson_value = Bson::I64(1234);
    let value = FieldType::get_value(&bson_value);
    assert_eq!(value, Some(ValueType::I64(1234)));
  }

  #[test]
  fn it_gets_value_floating_point() {
    let bson_value = Bson::FloatingPoint(1.2);
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
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
    field_type.set_probability(10);
    assert_eq!(field_type.probability, 0.1);
  }

  #[test]
  fn it_gets_unique() {
    let mut field_type =
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
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
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
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
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
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
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
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
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
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
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
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
    let bson_value = Bson::I32(1234);
    let mut field_type =
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
    field_type.update_value(&bson_value);
    assert_eq!(field_type.values[0], ValueType::I32(1234));
  }

  // #[bench]
  // fn bench_it_updates_value_some(bench: &mut Bencher) {
  //   let bson_value = Bson::I32(1234);
  //   let mut field_type =
  //     FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
  //   bench.iter(|| field_type.update_value(&bson_value));
  // }
}
