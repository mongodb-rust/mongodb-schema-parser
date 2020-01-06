use super::{Bson, FieldType};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Field {
  pub name: String,
  pub path: String,
  pub count: usize,
  pub bson_types: Vec<String>,
  pub probability: f32,
  pub types: HashMap<String, FieldType>,
}

impl Field {
  pub fn new<T, U>(name: T, path: U) -> Self
  where
    T: Into<String>,
    U: Into<String>,
  {
    Field {
      name: name.into(),
      count: 1,
      path: path.into(),
      bson_types: Vec::new(),
      probability: 0.0,
      types: HashMap::new(),
    }
  }

  pub fn create_type(&mut self, value: &Bson) {
    let mut field_type = FieldType::new(&self.path, &FieldType::get_type(&value));
    field_type.add_to_type(&value, self.count);
    self.bson_types.push(field_type.bson_type.to_string());
    self
      .types
      .insert(FieldType::get_type(&value), field_type.to_owned());
  }

  pub fn does_field_type_exist(&mut self, value: &Bson) -> bool {
    self.bson_types.contains(&FieldType::get_type(&value))
  }

  pub fn get_path(name: String, path: Option<String>) -> String {
    match path {
      None => name,
      Some(mut path) => {
        path.push_str(".");
        path.push_str(&name);
        path
      }
    }
  }

  pub fn finalise_field(&mut self, parent_count: usize) {
    self.set_probability(parent_count);
    for field_type in self.types.values_mut() {
      field_type.finalise_type(self.count);
    }
  }

  pub fn update_for_missing(&mut self, missing: usize) {
    // create new field_types of "Null" for missing fields.
    let mut null_field_type = FieldType::new(&self.path, &FieldType::get_type(&Bson::Null));
    null_field_type.add_to_type(&Bson::Null, self.count);
    null_field_type.count = missing;
    self.types.insert(
      crate::field_type::NULL.to_string(),
      null_field_type.to_owned(),
    );
    self.bson_types.push(null_field_type.bson_type);
    // need to update internal field count, since otherwise on the next
    // iteration we will get integer overflow
    self.update_count_by(missing);
  }

  pub fn update_count(&mut self) {
    self.count += 1
  }

  fn update_count_by(&mut self, num: usize) {
    self.count += num
  }

  #[inline]
  fn set_probability(&mut self, parent_count: usize) {
    self.probability = self.count as f32 / parent_count as f32
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  // use crate::test::Bencher;

  #[test]
  fn it_creates_new() {
    let path = "Nori.cat";
    let count = 1;

    let field = Field::new("Nori", path);

    assert_eq!(field.name, "Nori".to_string());
    assert_eq!(field.path, path);
    assert_eq!(field.count, count);
  }

  // #[bench]
  // fn bench_it_creates_new(bench: &mut Bencher) {
  //   let path = "Nori.cat";

  //   bench.iter(|| Field::new("Nori", &path));
  // }

  #[test]
  fn it_gets_path_if_none() {
    let path = Field::get_path(String::from("address"), None);
    assert_eq!(path, String::from("address"));
  }

  #[test]
  fn it_gets_path_if_some() {
    let path = Field::get_path(
      String::from("postal_code"),
      Some(String::from("address")),
    );
    assert_eq!(path, String::from("address.postal_code"));
  }

  // #[bench]
  // fn bench_it_gets_path(bench: &mut Bencher) {
  //   bench.iter(|| {
  //     Field::get_path(
  //       String::from("postal_code"),
  //       Some(String::from("address")),
  //     )
  //   });
  // }

  #[test]
  fn it_updates_count() {
    let mut field = Field::new("Chashu", "Chashu.cat");
    field.update_count();
    assert_eq!(field.count, 2);
  }

  // #[bench]
  // fn bench_it_updates_count(bench: &mut Bencher) {
  //   let mut field = Field::new("Chashu", "Chashu.cat");
  //   bench.iter(|| field.update_count());
  // }

  #[allow(clippy::float_cmp)]
  #[test]
  fn it_sets_probability() {
    let mut field = Field::new("Nori", "Nori.cat");
    field.set_probability(10);
    assert_eq!(field.probability, 0.1);
  }
}
