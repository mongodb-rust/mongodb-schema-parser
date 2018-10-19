#[derive(Debug, Clone)]
pub struct MongoDBSchema {
  pub(crate) count: usize,
  pub(crate) fields: Vec<Field>,
}

// TODO: field_type should be able to be either string or a vector of strings
#[derive(Debug, Clone)]
pub struct Field {
  pub(crate) name: String,
  pub(crate) path: String,
  pub(crate) count: usize,
  pub(crate) field_type: String,
  pub(crate) probability: f64,
  pub(crate) has_duplicates: bool,
  pub(crate) types: Vec<DocumentKind>,
}

#[derive(Debug, Clone)]
pub struct ConstantType {
  name: String,
  path: String,
  count: usize,
  probability: f64,
  has_duplicates: bool,
  unique: usize,
}

// TODO: values should be able to be a vector of either strings, vectors, or
// booleans.
#[derive(Debug, Clone)]
pub struct PrimitiveType {
  pub(crate) name: String,
  pub(crate) path: String,
  pub(crate) count: usize,
  pub(crate) probability: f64,
  pub(crate) values: Vec<usize>,
  pub(crate) has_duplicates: bool,
  pub(crate) unique: usize,
}

#[derive(Debug, Clone)]
pub struct ArrayType {
  name: String,
  path: String,
  lengths: Vec<usize>,
  average_length: usize,
  total_count: usize,
  count: usize,
  probability: f64,
  has_duplicates: bool,
  unique: usize,
}

#[derive(Debug, Clone)]
pub struct DocumentType {
  name: String,
  path: String,
  count: usize,
  probability: f64,
  has_duplicates: bool,
  unique: usize,
  fields: Vec<String>,
}

/// each Field can have a vector of documents, and they can be either:
#[derive(Debug, Clone)]
pub enum DocumentKind {
  PrimitiveType(PrimitiveType),
  ConstantType(ConstantType),
  DocumentType(DocumentType),
  ArrayType(ArrayType),
}

/// Field type present in a given schema: can be either a vector or a Stringing
#[derive(Debug, Clone)]
pub enum FieldKind {
  Vec,
  String,
}

#[derive(Debug, Clone)]
pub enum PrimitiveKind {
  String,
  usize,
  bool,
}
