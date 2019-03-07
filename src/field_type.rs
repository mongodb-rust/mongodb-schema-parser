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
          let value_type = Self::get_value(val);
          if let Some(value_type) = value_type {
            self.values.push(value_type);
          }
        }
        self
      }
      Bson::Document(subdoc) => {
        let mut schema_parser = SchemaParser::new();
        schema_parser.generate_field(
          subdoc.to_owned(),
          Some(self.path.clone()),
          Some(&self.count),
        );
        self.set_schema(schema_parser);
        self
      }
      _ => {
        let value_type = Self::get_value(&bson_value);
        if let Some(value_type) = value_type {
          self.values.push(value_type);
        }
        self
      }
    }
  }

  pub fn update_type(&mut self, value: &Bson) {
    let bson_type = self.bson_type.clone();
    let path = self.path.clone();

    if &bson_type == "Document" {
      match &mut self.schema {
        Some(schema_parser) => match &value {
          Bson::Document(subdoc) => schema_parser.generate_field(
            subdoc.to_owned(),
            Some(path),
            Some(&self.count),
          ),
          _ => unimplemented!(),
        },
        None => unimplemented!(),
      }
    }

    self.update_count();
    self.update_value(&value);
  }

  pub fn get_value(value: &Bson) -> Option<ValueType> {
    match value {
      Bson::FloatingPoint(num) => Some(ValueType::FloatingPoint(*num)),
      Bson::String(string) => Some(ValueType::Str(string.to_string())),
      Bson::Boolean(boolean) => Some(ValueType::Boolean(*boolean)),
      Bson::ObjectId(id) => Some(ValueType::Str(id.to_string())),
      Bson::I32(num) => Some(ValueType::I32(*num)),
      Bson::I64(num) => Some(ValueType::I64(*num)),
      _ => None,
    }
  }

  pub fn finalise_type(&mut self, parent_count: usize) {
    self.set_probability(parent_count);
    self.set_unique();
    self.set_duplicates();
  }

  pub fn get_type(value: &Bson) -> String {
    match value {
      Bson::FloatingPoint(_) | Bson::I32(_) | Bson::I64(_) => {
        "Number".to_string()
      }
      Bson::Document(_) => "Document".to_string(),
      Bson::ObjectId(_) => "ObjectId".to_string(),
      Bson::Boolean(_) => "Boolean".to_string(),
      Bson::String(_) => "String".to_string(),
      Bson::Array(_) => "Array".to_string(),
      Bson::Null => "Null".to_string(),
      _ => unimplemented!(),
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

  fn update_value(&mut self, value: &Bson) {
    match value {
      Bson::Array(arr) => {
        for val in arr.iter() {
          let value_type = Self::get_value(val);

          if let Some(value_type) = value_type {
            self.values.push(value_type)
          }
        }
      }
      _ => {
        let value_type = Self::get_value(&value);
        if let Some(value_type) = value_type {
          self.values.push(value_type)
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test::Bencher;

  #[test]
  fn it_creates_new() {
    let address = "address";
    let bson_string = Bson::String("Oranienstr. 123".to_string());
    let field_type = FieldType::new(address, &bson_string);
    assert_eq!(field_type.path, address);
  }

  #[bench]
  fn bench_it_creates_new(bench: &mut Bencher) {
    bench.iter(|| {
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()))
    });
  }

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

  #[test]
  fn it_gets_value_none() {
    let bson_value = Bson::TimeStamp(1234);
    let value = FieldType::get_value(&bson_value);
    assert_eq!(value, None);
  }

  #[bench]
  fn bench_it_gets_value(bench: &mut Bencher) {
    let bson_value = Bson::String("cats".to_string());
    bench.iter(|| FieldType::get_value(&bson_value));
  }

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

  #[bench]
  fn bench_it_gets_unique(bench: &mut Bencher) {
    let mut field_type =
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
    field_type.values.push(ValueType::Str("Berlin".to_string()));
    field_type
      .values
      .push(ValueType::Str("Hamburg".to_string()));
    bench.iter(|| field_type.get_unique());
  }

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

  #[bench]
  fn bench_it_sets_unique(bench: &mut Bencher) {
    let mut field_type =
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
    field_type.values.push(ValueType::Str("Berlin".to_string()));
    field_type
      .values
      .push(ValueType::Str("Hamburg".to_string()));
    bench.iter(|| field_type.set_unique());
  }

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

  #[bench]
  fn bench_it_gets_duplicates(bench: &mut Bencher) {
    let mut field_type =
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
    field_type.values.push(ValueType::Str("Berlin".to_string()));
    field_type
      .values
      .push(ValueType::Str("Hamburg".to_string()));
    bench.iter(|| field_type.get_duplicates());
  }

  #[test]
  fn it_sets_duplicates() {
    let mut field_type =
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
    field_type.values.push(ValueType::Str("Berlin".to_string()));
    field_type.values.push(ValueType::Str("Berlin".to_string()));
    field_type.set_duplicates();
    assert_eq!(field_type.has_duplicates, true)
  }

  #[bench]
  fn bench_it_sets_duplicates(bench: &mut Bencher) {
    let mut field_type =
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
    bench.iter(|| field_type.set_duplicates());
  }

  #[test]
  fn it_updates_count() {
    let mut field_type =
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
    field_type.update_count();
    assert_eq!(field_type.count, 2);
  }

  #[bench]
  fn bench_it_updates_count(bench: &mut Bencher) {
    let mut field_type =
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
    bench.iter(|| field_type.update_count());
  }

  #[test]
  fn it_updates_value_some() {
    let bson_value = Bson::I32(1234);
    let mut field_type =
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
    field_type.update_value(&bson_value);
    assert_eq!(field_type.values[0], ValueType::I32(1234));
  }

  #[bench]
  fn bench_it_updates_value_some(bench: &mut Bencher) {
    let bson_value = Bson::I32(1234);
    let mut field_type =
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
    bench.iter(|| field_type.update_value(&bson_value));
  }

  #[test]
  fn it_updates_value_none() {
    let bson_value = Bson::TimeStamp(1234);
    let mut field_type =
      FieldType::new("address", &Bson::String("Oranienstr. 123".to_string()));
    field_type.update_value(&bson_value);
    assert!(field_type.values.is_empty());
  }
}
