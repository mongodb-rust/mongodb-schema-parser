# mongodb-schema-parser
[![crates.io version][1]][2] [![build status][3]][4]
[![downloads][5]][6] [![docs.rs docs][7]][8]

Infer a probabilistic schema for a MongoDB collection. This library is meant
to be used in Rust or as Web Assembly module in JavaScript.

- [Documentation][8]
- [Crates.io][2]
- [Rust API](https://github.com/mongodb-rust/mongodb-schema-parser#rust-api)
- [JavaScript API](https://github.com/mongodb-rust/mongodb-schema-parser#javascript-api)
- [npm package][9]

# Usage: in Rust
```rust
use SchemaParser

pub fn main () {
  let mut file = fs::read_to_string("examples/fanclub.json").unwrap();
  let file: Vec<&str> = file.split("\n").collect();
  let schema_parser = SchemaParser::new();
  for json in file {
    schema_parser.write(&json)?;
  }
  let result = schema_parser.flush();
  println!("{:?}", result);
}
```
## Rust API:
### `schema_parser = SchemaParser::new() -> Self`
Creates a new SchemaParser instance. 

### `schema_parser.write_bson(doc: Document) -> Result((), failure::Error)`
Start populating instantiated schema_parser with [Bson OrderedDocument](https://docs.rs/bson/0.13.0/bson/ordered/struct.OrderedDocument.html). This should be called for each document you add:
```rust
use bson::{doc, bson};
let schema_parser = SchemaParser::new()
schema_parser.write_bson(doc! {"name": "Nori", "type": "Norwegian Forest Cat"});
schema_parser.write_bson(doc! {"name": "Rey", "type": "Viszla"});
```

### `schema_parser.write_json(json: &str) -> Result((), failure::Error)`
Start populating instantiated schema_parser with a string slice. This should also be called individually for each document:

```rust
let schema_parser = SchemaParser::new()
schema_parser.write_json(r#"{"name": "Chashu", "type": "Norwegian Forest Cat"}"#);
schema_parser.write_bson(r#"{"name": "Rey", "type": "Viszla"}"#);
```

### `schema_parser.flush() -> SchemaParser`
Internally this finalizes the output schema with missing fields, duplicates
and probability calculations. SchemaParser is ready to be used after this
step.

### `schema_parser.to_json() -> Result(String, failure::Error)`
Returns a serde serialized version of the resulting struct. Before using
`.to_json()`, a `.flush()` should be called to finalize schema.


# Usage: in JavaScript 
Make sure your environment is setup for WebAssembly usage. Check out
[CONTRIBUTING.md](./CONTRIBUTING.md) for more detailed instructions.

```js
var schemaWasm = import('@mongodb-rust/wasm-schema-parser')

schemaWasm.then(module => {
  var schemaParser = new module.SchemaParser()
  try {
    schemaParser.writeJson('{"name": "Chashu", "type": "Norwegian Forest Cat"}')
  } catch (e) {
    throw new Error("schema-parser: Could not write Json", e)
  }
  var result = schemaParser.toObject()
  console.log(result)
})
.catch(e => console.error('Cannot load @mongodb-rust/wasm-schema-parser', e))
```

## JavaScript API:

### `schemaParser = new SchemaParser()`
Creates a new SchemaParser instance.

### `schemaParser.writeRaw(bsonBuf)`
Writes a document in raw `BSON` buffer form to Schema Parser. This buffer can be obtained from MongoDB by passing the `raw` flag to node driver. 

### `schemaParser.writeJson(json)`
Writes a document in a form of `json` string to SchemaParser.

### `schema = schemaParser.toJson()`
Returns parsed schema in `json` form.

### `schema = schemaParser.toObject()`
Returns parsed schema as a JavaScript Object. Eliminates the need to call
`JSON.parse()` on a JSON string.

## Installation
```sh
$ cargo add mongodb-schema-parser 
```

## License
[Apache-2.0](./LICENSE)

[1]: https://img.shields.io/crates/v/mongodb-schema-parser.svg?style=flat-square
[2]: https://crates.io/crates/mongodb-schema-parser
[3]: https://img.shields.io/travis/mongodb-rust/mongodb-schema-parser.svg?style=flat-square
[4]: https://travis-ci.org/mongodb-rust/mongodb-schema-parser
[5]: https://img.shields.io/crates/d/mongodb-schema-parser.svg?style=flat-square
[6]: https://crates.io/crates/mongodb-schema-parser
[7]: https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square
[8]: https://docs.rs/mongodb-schema-parser
[9]: https://npmjs.com/@mongodb-rust/wasm-schema-parser
