#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum ValueType {
  Str(String),
  I32(i32),
  I64(i64),
  FloatingPoint(f64),
  Boolean(bool),
}
