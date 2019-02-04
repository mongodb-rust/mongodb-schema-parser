# mongodb-schema-parser
[![crates.io version][1]][2] [![build status][3]][4]
[![downloads][5]][6] [![docs.rs docs][7]][8]

Infer a probabilistic schema for a MongoDB collection. This library is meant
to be used in Rust or as Web Assembly module in JavaScript.

- [Documentation][8]
- [Crates.io][2]

## Usage: in Rust
```rust
use SchemaParser

pub fn main () {
  let mut file = fs::read_to_string("examples/fanclub.json").unwrap();
  let file: Vec<&str> = file.split("\n").collect();
  let schema_parser = SchemaParser::new();
  for json in file {
    schema_parser.write(&json)?;
  }
  let result = schema_parser.read();
  println!("{:?}", result);
}
```

## Usage: in JavaScript 
Make sure your environment is setup for Web Assembly usage. 
```js
import { SchemaParser } from "mongodb-schema-parser";

const schemaParser = new SchemaParser()

// get the json file
fetch('./fanclub.json')
  .then(response => response.text())
  .then(data => {
    var json = data.split("\n")
    for (var i = 0; i < json.length; i++) {
      if (json[i] !== '') {
        // feed the parser json line by line
        schemaParser.write(json[i])
      }
    }
    // get the result as a json string
    var result = schemaParser.toJson()
    console.log(result)
  })
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
