use super::{SchemaParser};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum ValueType {
  Str(String),
  I32(i32),
  I64(i64),
  Decimal128(String),
  FloatingPoint(f64),
  Document(SchemaParser),
  Binary(Vec<u8>),
  Boolean(bool),
  Null(String),
}
