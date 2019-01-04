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
