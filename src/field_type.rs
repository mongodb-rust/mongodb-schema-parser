use super::{Bson, ValueType};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[allow(non_snake_case)]
pub struct FieldType {
  pub name: Option<String>,
  pub path: String,
  pub count: usize,
  pub bsonType: Option<String>,
  pub probability: Option<f64>,
  pub values: Vec<ValueType>,
  pub has_duplicates: Option<bool>,
  pub unique: Option<usize>,
}

impl FieldType {
  pub fn new(path: &str) -> Self {
    FieldType {
      name: None,
      path: path.to_string(),
      bsonType: None,
      count: 0,
      probability: None,
      values: Vec::new(),
      has_duplicates: None,
      unique: None,
    }
  }

  pub fn add_to_type(mut self, value: &Bson) -> Option<Self> {
    let bson_value = value.clone();
    let mut value_vec = Vec::new();

    match value {
      Bson::Array(arr) => {
        let bson_type = Self::get_type(&bson_value);
        self.set_name(bson_type.clone());
        self.set_bson_type(bson_type.clone());
        // add values item in array as a separate func;
        for val in arr.iter() {
          let value_type = Self::get_value(val);

          if let Some(value_type) = value_type {
            value_vec.push(value_type)
          }
        }
        self.set_values(value_vec);
        Some(self)
      }
      _ => {
        let value_type = Self::get_value(&bson_value);
        let bson_type = Self::get_type(&bson_value);
        self.set_name(bson_type.clone());
        self.set_bson_type(bson_type.clone());
        // add values item in array as a separate func;
        if let Some(value_type) = value_type {
          value_vec.push(value_type);
          self.set_values(value_vec);
        }
        Some(self)
      }
    }
  }

  pub fn get_value(value: &Bson) -> Option<ValueType> {
    match value {
      Bson::FloatingPoint(num) => Some(ValueType::FloatingPoint(*num)),
      Bson::Boolean(boolean) => Some(ValueType::Boolean(*boolean)),
      Bson::String(string) => Some(ValueType::Str(string.to_string())),
      Bson::I32(num) => Some(ValueType::I32(*num)),
      Bson::I64(num) => Some(ValueType::I64(*num)),
      _ => None,
    }
  }

  pub fn get_type(value: &Bson) -> Option<String> {
    match value {
      Bson::FloatingPoint(_) | Bson::I32(_) | Bson::I64(_) => {
        Some(String::from("Number"))
      }
      Bson::Document(_) => Some(String::from("Document")),
      Bson::Boolean(_) => Some(String::from("Boolean")),
      Bson::String(_) => Some(String::from("String")),
      Bson::Array(_) => Some(String::from("Array")),
      Bson::Null => Some(String::from("Null")),
      _ => None,
    }
  }

  pub fn set_name(&mut self, name: Option<String>) {
    self.name = name
  }

  pub fn set_bson_type(&mut self, bsontype: Option<String>) {
    self.bsonType = bsontype
  }

  pub fn update_count(&mut self) {
    self.count += 1
  }

  pub fn update_value(&mut self, value: &Bson) {
    let value_type = Self::get_value(&value);
    if let Some(value_type) = value_type {
      self.push_value(value_type)
    }
  }

  fn set_values(&mut self, values: Vec<ValueType>) {
    self.values = values
  }

  fn push_value(&mut self, value: ValueType) {
    self.values.push(value)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::test::Bencher;

  #[test]
  fn it_creates_new() {
    let address = "address";
    let field_type = FieldType::new(address);
    assert_eq!(field_type.path, address);
  }

  #[bench]
  fn bench_it_creates_new(bench: &mut Bencher) {
    bench.iter(|| FieldType::new("address"));
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

  #[test]
  fn it_sets_type() {
    let mut field_type = FieldType::new("address");
    field_type.set_name(Some("postal_code".to_string()));
    assert_eq!(field_type.name, Some("postal_code".to_string()));
  }

  #[bench]
  fn bench_it_sets_type(bench: &mut Bencher) {
    let mut field_type = FieldType::new("address");
    bench.iter(|| field_type.set_name(Some("postal_code".to_string())));
  }

  #[test]
  fn it_sets_bson_type() {
    let mut field_type = FieldType::new("address");
    field_type.set_bson_type(Some("Document".to_string()));
    assert_eq!(field_type.bsonType, Some("Document".to_string()));
  }

  #[bench]
  fn bench_it_sets_bson_type(bench: &mut Bencher) {
    let mut field_type = FieldType::new("address");
    bench.iter(|| field_type.set_bson_type(Some("Document".to_string())));
  }

  #[test]
  fn it_updates_count() {
    let mut field_type = FieldType::new("address");
    field_type.update_count();
    assert_eq!(field_type.count, 1);
  }

  #[bench]
  fn bench_it_updates_count(bench: &mut Bencher) {
    let mut field_type = FieldType::new("address");
    bench.iter(|| field_type.update_count());
  }

  #[test]
  fn it_updates_value_some() {
    let bson_value = Bson::I32(1234);
    let mut field_type = FieldType::new("address");
    field_type.update_value(&bson_value);
    assert_eq!(field_type.values[0], ValueType::I32(1234));
  }

  #[bench]
  fn bench_it_updates_value_some(bench: &mut Bencher) {
    let bson_value = Bson::I32(1234);
    let mut field_type = FieldType::new("address");
    bench.iter(|| field_type.update_value(&bson_value));
  }

  #[test]
  fn it_updates_value_none() {
    let bson_value = Bson::TimeStamp(1234);
    let mut field_type = FieldType::new("address");
    field_type.update_value(&bson_value);
    assert!(field_type.values.is_empty());
  }

  #[test]
  fn it_sets_value() {
    let mut field_type = FieldType::new("address");
    let vec = vec![ValueType::I32(1234), ValueType::I64(1234)];
    field_type.set_values(vec.clone());
    assert_eq!(&field_type.values, &vec)
  }

  #[bench]
  fn bench_it_sets_value(bench: &mut Bencher) {
    let mut field_type = FieldType::new("address");
    bench.iter(|| {
      let vec = vec![ValueType::I32(1234), ValueType::I64(1234)];
      let n = crate::test::black_box(vec);
      field_type.set_values(n)
    });
  }

  #[test]
  fn it_pushes_value() {
    let value_type = ValueType::I32(1234);
    let mut field_type = FieldType::new("address");
    field_type.push_value(value_type.clone());
    assert_eq!(field_type.values[0], value_type);
  }

  #[bench]
  fn bench_it_pushes_value(bench: &mut Bencher) {
    let mut field_type = FieldType::new("address");
    bench.iter(|| {
      let value_type = ValueType::I32(1234);
      let n = crate::test::black_box(value_type);
      field_type.push_value(n)
    });
  }
}
