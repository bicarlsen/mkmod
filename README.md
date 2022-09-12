# mkmod

### Easily add modules to a Rust project.

Creates a new module in a Rust project.
This is done by creating a file or directory based on the name of the module
provided.
The module can include a seperate test file, and be automatically added to its
partent module.

## Install

### Cargo (recommended)
> This requires [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) to be installed.

Run `cargo +nightly install mkmod` from your terminal.

### Manual
Download the `mkmod` executable from the desired release and add it to your path.

## Examples

### File module
```bash
mkmod new_mod
```
Adds a new file module called `new_mod` to the current directory.

This will add the files `new_mod.rs` and `new_mod_test.rs` to the directory.
`new_mod.rs` will contain testing boilerplate pointing to the `new_mod_test.rs`
file.

`new_mod` will also be added as a public module to its parent.

### Directory module
```bash
mkmod big_mod --dir
```
Adds a new directory module named `big_mod` to the current directory.

This will add a directory called `big_mod` to the current directory with files
`mod.rs` and `mod_test.rs`.
`mod.rs` will have testing boilerplate pointing to the `mod_test.rs` file.

### Root module
```bash
mkmod my_mod --main
```
By default, modules added to the root directory will first try to be added
to `lib.rs`. If `lib.rs` does not exist, they will then attempt to be added to `main.rs`.
You can force a module to be added to `main.rs` using the `--main` flag.

### Misc.
```bash
mkmod path/to/my_mod
```

```bash
mkmod my_mod --no-test
```

```bash
mkmod my_mod --no-add
```

```bash
mkmod my_mod --private
```
