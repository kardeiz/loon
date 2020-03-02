# loon

[![Docs](https://docs.rs/loon/badge.svg)](https://docs.rs/crate/loon/)
[![Crates.io](https://img.shields.io/crates/v/loon.svg)](https://crates.io/crates/loon)

### lo[calizati]on

A very simple localization/internationalization provider, inspired by `ruby-i18n`.

### Usage:

```rust
fn main() {

    loon::set_config(loon::Config::default().with_path_pattern("examples/locales/en.yml")).unwrap();

    assert_eq!(loon::t("/greeting", None).unwrap(), String::from("Hello, World!"));
}
```

<hr/>

Current version: 0.1.0

License: MIT
