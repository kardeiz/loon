# loon

[![Docs](https://docs.rs/loon/badge.svg)](https://docs.rs/loon/)
[![Crates.io](https://img.shields.io/crates/v/loon.svg)](https://crates.io/crates/loon)

<h2>lo<span style="color:Silver;">calizati</span>on</h2>

A very simple localization/internationalization provider, inspired by `ruby-i18n`.

### Usage:

```rust
fn main() {

    use loon::*;

    set_config(PathPattern("examples/locales/*.yml")).unwrap();

    assert_eq!(
        t("custom.greeting", Var("name", "Jacob")).unwrap(),
        String::from("Hello, Jacob!!!")
    );

    assert_eq!(
        t("greeting", Opts::default().locale("de")).unwrap(),
        String::from("Hallo Welt!")
    );
}
```

### Features

Translation files can be:
* JSON
* YAML (enabled by default, disable with `default-features = false`), or
* TOML (enable with `features = ["toml"]`).

<hr/>

Current version: 0.3.0

License: MIT
