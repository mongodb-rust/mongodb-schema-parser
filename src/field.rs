use super::FieldType;

#[derive(Serialize, Deserialize, Debug, Clone)]
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
  pub fn new(name: &str, path: String, count: usize) -> Self {
    Field {
      name: name.to_string(),
      count,
      path,
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

// #[cfg(test)]
//
// mod tests {
//   use super::*;
//
//   #[test]
//   fn new() {
//     let name = "Nori";
//     let path = String.from("Nori.cat");
//     let count = 1;
//
//     let preset_field = Field {
//       name: "Nori",
//       count: 1,
//       path: "Nori.cat",
//       filed_type: None,
//       probability: None,
//       has_duplicates: None,
//       types: Vec::new(),
//     };
//     let new_field = Field::new(name, path, count);
//
//     // assert_eq!(preset_field, new_field);
//   }
// }
