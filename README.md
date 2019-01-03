# mongodb-schema-parser
[![crates.io version][1]][2] [![build status][3]][4]
[![downloads][5]][6] [![docs.rs docs][7]][8]

MongoDB Schema Parser.

- [Documentation][8]
- [Crates.io][2]

## Usage
```rust
use SchemaParser

pub fn main () {
  let mut file = fs::read_to_string("examples/fanclub.json").unwrap();
  let file: Vec<&str> = file.split("\n").collect();
  let schema_parser = SchemaParser::new();
  for json in file {
    schema_parser.write(&json)?;
  }
  let result = schema_parser.to_json();
}
```

## Installation
```sh
$ cargo add mongodb-schema-parser 
```

## License
[Apache-2.0](./LICENSE)

[1]: https://img.shields.io/crates/v/mongodb-schema-parser.svg?style=flat-square
[2]: https://crates.io/crates/mongodb-schema-parser
[3]: https://img.shields.io/travis/lrlna/mongodb-schema-parser.svg?style=flat-square
[4]: https://travis-ci.org/lrlna/mongodb-schema-parser
[5]: https://img.shields.io/crates/d/mongodb-schema-parser.svg?style=flat-square
[6]: https://crates.io/crates/mongodb-schema-parser
[7]: https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square
[8]: https://docs.rs/mongodb-schema-parser
