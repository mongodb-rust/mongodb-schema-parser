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
  * [Wasm Build](#wasm-build)
  * [Coding Style](#coding-style)

- [Code of Conduct](#code-of-conduct)

- [Bad Actors](#bad-actors)

## Environment Setup
### Latest Rust
This project uses a few things that you should set yourself up with before
starting work. First of all, make sure you have the latest `rust` and `cargo`
installed. The best way to do that is with `rustup`, and you can read about it
more in the [rust
book](https://doc.rust-lang.org/book/second-edition/ch01-01-installation.html).
But to get yourself uptodate with all the things:

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
[vscode](https://github.com/rust-lang-nursery/rls-vscode).

`rustfmt` will also run as a pre-commit hook. You will need to copy the file
that's currently in `./hooks/pre-commit` to your local `.git` directory:
```bash
cp hooks/pre-commit ./git/hooks/pre-commit
```

### Wasm Build
To be able to use this module in JavaScript and Node, we compile it to WASM. For
that we use [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) inside our
`lib.rs`, and [wasm-pack](https://github.com/rustwasm/wasm-pack) to make a
package to be published to `npm`. To do so, install `wasm-pack`:
```bash
cargo install wasm-pack
```
and run build that will generate a `pkg` directory that can be then published to
npm:
```bash
wasm-pack build
```
Travis CI will also run a wasm-pack build to check we are able to compile this correctly.

### Coding Style

A few things to follow when working on this project. 

1. Avoid using `unsafe-rust`. `lib.rs` is already setup with
   `#[deny(unsafe-rust)]` to help with that.

2. Structs should implement `Copy` and `Debug` traits to avoid future
   complications. These can be simple appended with:
``` rust
#[derive(Debug, Copy)]
struct Pair(Box<i32>, Box<i32>)
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
