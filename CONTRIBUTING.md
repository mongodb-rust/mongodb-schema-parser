# Contributing
Contributions include code, documentation, answering user questions, running the
project's infrastructure, and advocating for all types of users.

The project welcomes all contributions from anyone willing to work in good faith
with other contributors and the community. No contribution is too small and all
contributions are valued.

This guide explains the process for contributing to the project's GitHub
Repository.

- [Environment Setup](#environment-setup)
  * [Latest Rust](#latest-rust)
  * [Formatting](#formatting)
  * [WASM Build](#wasm-build)
  * [WASM in JS](#wasm-in-js)

- [Code of Conduct](#code-of-conduct)

- [Bad Actors](#bad-actors)

## Environment Setup
### Latest Rust
This project uses a few things that you should set yourself up with before
starting work. First of all, make sure you have the latest `rust` and `cargo`
installed. The best way to do that is with `rustup`, and you can read about
it more in the [rust
book](https://doc.rust-lang.org/book/ch01-01-installation.html). Similarly
working with cargo is better described in [the
book](https://doc.rust-lang.org/book/ch01-03-hello-cargo.html)

To get yourself uptodate with all the things:
```bash
rustup update
```

### Formatting
We are using two tools to help with best-practice fromatting:
[rustfmt](https://github.com/rust-lang-nursery/rustfmt) and
[clippy](https://github.com/rust-lang-nursery/rust-clippy). There is already an
existing `rustfmt.toml` for the project, and you can also setup your editor to
autoformat. If you're on `vim`, something like this is really helpful in your
`.vimrc`:

```vim
" rust plugin
Plugin 'rust-lang/rust.vim'

" autoformatting
au BufRead,BufNewFile *.rs set filetype=rust
let g:rustfmt_autosave = 1
let g:rust_recommended_style = 0
augroup filetype_rust
  autocmd!
  autocmd BufReadPost *.rs setlocal filetype=rust
  setl sw=2 sts=2 et
augroup END
```
`rust-nursery` also has support for other editors, like
[vscode](https://github.com/rust-lang-nursery/rls-vscode). You can just add
this plugin to your VSCode setup, and magic :sparkles:, everything works!

### WASM Build
To be able to use this module in JavaScript and Node, we compile it to WASM. For
that we use [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) inside our
`lib.rs`, and [wasm-pack](https://github.com/rustwasm/wasm-pack) to make a
package to be published to `npm`. To do so, install `wasm-pack`:
```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

and run build that will generate a `pkg` directory that can be then published to
npm:

```bash
wasm-pack build --no-typescript --release --scope=mongodb-rust
```

If you are developing locally, you can also point your javascript imports to
the build. For example:
```js
var schemaWasm = import('../../mongodb-schema-parser/pkg')
```

To publish you can also use `wasm-pack`:
```bash
wasm-pack publish --access=public
``` 

As you are developing sometimes it's useful to know whether you can compile
to WASM. Especially when you add a new crate to the setup:
```bash
cargo check --target wasm32-unknown-unknown
```

Travis CI will also run a wasm-pack build to check we are able to compile this correctly.

### WASM in JS
The easiest way to run WASM in the browser is via
[webpack](https://webpack.js.org/). If you are running this in electron, it's
recommended to be on webpack > `4.29.6`. There have been a few bugs that were
fixed in that version, and we find it to be quite stable. If you're in the
browser-browser and not in electron, anything webpack > 4.x.

You will need to add `.wasm` to your [resolve extensions](https://webpack.js.org/configuration/resolve/#resolve-extensions):
```js
extensions: ['.js', '.jsx', '.json', 'less', '.wasm']
```

WASM _needs_ to be loaded async and dynamically. The easiest way to do this
is to have a [babel
plugin](https://www.npmjs.com/package/babel-plugin-syntax-dynamic-import):
```js
// in .babelrc
{
  "plugins": [
    "syntax-dynamic-import"
  ]
}
```

After, you can just import your plugin, wrap its loading in a promise and use the API as intended:
```js
// note the import rather than require!
var wasm = import('wasm')
function runWasmAction (param) {
  // .then on the previously imported module
  wasm
    .then(module => {
      // use your module's API
      new module.ConstructorMethod()
    })
    .catch(e => return new Error(`Error in wasm module ${e}`))
}
```

## Code of Conduct
The project has a [Code of Conduct](./CODE_OF_CONDUCT.md) that *all*
contributors are expected to follow. This code describes the *minimum* behavior
expectations for all contributors.

As a contributor, how you choose to act and interact towards your
fellow contributors, as well as to the community, will reflect back not only
on yourself but on the project as a whole. The Code of Conduct is designed and
intended, above all else, to help establish a culture within the project that
allows anyone and everyone who wants to contribute to feel safe doing so.

Should any individual act in any way that is considered in violation of the
[Code of Conduct](./CODE_OF_CONDUCT.md), corrective actions will be taken. It is
possible, however, for any individual to *act* in such a manner that is not in
violation of the strict letter of the Code of Conduct guidelines while still
going completely against the spirit of what that Code is intended to accomplish.

Open, diverse, and inclusive communities live and die on the basis of trust.
Contributors can disagree with one another so long as they trust that those
disagreements are in good faith and everyone is working towards a common
goal.

## Bad Actors
All contributors to tacitly agree to abide by both the letter and
spirit of the [Code of Conduct](./CODE_OF_CONDUCT.md). Failure, or
unwillingness, to do so will result in contributions being respectfully
declined.

A *bad actor* is someone who repeatedly violates the *spirit* of the Code of
Conduct through consistent failure to self-regulate the way in which they
interact with other contributors in the project. In doing so, bad actors
alienate other contributors, discourage collaboration, and generally reflect
poorly on the project as a whole.

Being a bad actor may be intentional or unintentional. Typically, unintentional
bad behavior can be easily corrected by being quick to apologize and correct
course *even if you are not entirely convinced you need to*. Giving other
contributors the benefit of the doubt and having a sincere willingness to admit
that you *might* be wrong is critical for any successful open collaboration.

Don't be a bad actor.
