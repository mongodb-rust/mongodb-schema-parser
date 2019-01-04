use super::FieldType;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Field {
  pub name: String,
  pub path: String,
  pub count: usize,
  pub field_type: Option<String>,
  pub probability: Option<f64>,
  pub has_duplicates: Option<bool>,
  pub types: Vec<FieldType>,
}

impl Field {
  pub fn new(name: &str, path: &str, count: usize) -> Self {
    Field {
      name: name.to_string(),
      count,
      path: path.to_string(),
      field_type: None,
      probability: None,
      has_duplicates: None,
      types: Vec::new(),
    }
  }

  pub fn add_to_types(&mut self, field_type: Option<FieldType>) {
    if let Some(field_type) = field_type {
      self.types.push(field_type)
    }
  }

  pub fn get_path(name: String, path: &Option<String>) -> String {
    match &path {
      None => name,
      Some(path) => {
        let mut path = path.clone();
        path.push_str(".");
        path.push_str(&name);
        path
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_creates_new() {
    let name = "Nori";
    let path = "Nori.cat";
    let count = 1;

    let field = Field::new(&name, &path, count);

    assert_eq!(field.name, name);
    assert_eq!(field.path, path);
    assert_eq!(field.count, count);
  }

  // #[test]
  // #[ignore]
  // fn it_adds_to_types() {
  //   let mut field = Field::new("Chashu", "Chashu.cat", 1);
  //   let field_type = FieldType::new("path");
  //   field.add_to_types(Some(field_type.clone()));
  //   assert_eq!(field.types[0], field_type);
  // }

  #[test]
  fn it_gets_path_if_none() {
    let path = Field::get_path(String::from("address"), &None);
    assert_eq!(path, String::from("address"));
  }

  #[test]
  fn it_gets_path_if_some() {
    let path = Field::get_path(
      String::from("postal_code"),
      &Some(String::from("address")),
    );
    assert_eq!(path, String::from("address.postal_code"));
  }
}
