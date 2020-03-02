/*!
## lo[calizati]on

A very simple localization/internationalization provider, inspired by `ruby-i18n`.

## Usage:

```rust
fn main() {
    
    loon::set_config(loon::Config::default().with_path_pattern("examples/locales/en.yml")).unwrap();

    assert_eq!(loon::t("/greeting", None).unwrap(), String::from("Hello, World!"));
}
```
*/

/// Error management
pub mod err {

    /// Error container
    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("IO error: `{0}`")]
        Io(#[from] std::io::Error),
        #[error("strfmt error: `{0}`")]
        Strfmt(#[from] strfmt::FmtError),
        #[error("YAML error: {0}")]
        Yaml(#[from] serde_yaml::Error),
        #[error("Error: {0}")]
        Custom(Box<str>),
        #[error("Unknown locale: {0}")]
        UnknownLocale(Box<str>),
        #[error("Unknown key: {0}")]
        UnknownKey(Box<str>),
    }

    /// Create a custom error.
    pub fn custom<T: std::fmt::Display>(t: T) -> Error {
        Error::Custom(t.to_string().into_boxed_str())
    }

    pub type Result<T> = std::result::Result<T, Error>;
}

use once_cell::sync::{Lazy, OnceCell};
use std::collections::HashMap;
use std::path::PathBuf;

/// Options (optional) for the `translate` call
#[derive(Default)]
pub struct Opts<'a> {
    default_key: Option<&'a str>,
    vars: Option<HashMap<String, String>>,
    locale: Option<&'a str>,
    count: Option<i32>,
}

impl<'a> Opts<'a> {
    /// If the key does not exist, fallback to using another key.
    pub fn default_key(mut self, default_key: &'a str) -> Self {
        self.default_key = Some(default_key);
        self
    }

    /// Set the locale for this `translate` call.
    pub fn locale(mut self, locale: &'a str) -> Self {
        self.locale = Some(locale);
        self
    }

    /// Set any variables to be inerpolated.
    pub fn vars<I: Into<String>, J: std::fmt::Display>(mut self, key: I, value: J) -> Self {
        let mut vars = self.vars.take().unwrap_or_else(HashMap::new);
        vars.insert(key.into(), value.to_string());
        self.vars = Some(vars);
        self
    }

    /// Set the `count` for this translation.
    ///
    /// Uses Rails style pluralization options: `zero`, `one`, `other`.
    pub fn count(mut self, count: i32) -> Self {
        self.count = Some(count);
        self.vars("count", count)
    }
}

/// Container for translation messages
#[derive(Debug)]
pub struct Dictionary {
    inner: HashMap<String, serde_json::Value>,
    default_locale: String,
}

impl Default for Dictionary {
    fn default() -> Self {
        Self { inner: HashMap::new(), default_locale: "en".into() }
    }
}

impl Dictionary {
    /// Translate a message.
    pub fn translate<'a, I: Into<Option<Opts<'a>>>>(
        &self,
        key: &str,
        opts: I,
    ) -> err::Result<String> {
        
        let opts = opts.into().unwrap_or_else(Opts::default);

        let mut key = key;

        let alt_key;

        match opts.count {
            Some(0) => {
                alt_key = format!("{}/zero", key);
                key = &alt_key;
            },
            Some(1) => {
                alt_key = format!("{}/one", key);
                key = &alt_key;
            },
            Some(_) => {
                alt_key = format!("{}/other", key);
                key = &alt_key;
            }
            _ => {}
        }

        let locale = opts.locale.unwrap_or_else(|| &self.default_locale);

        let localized = self
            .inner
            .get(locale)
            .ok_or_else(|| err::Error::UnknownLocale(String::from(locale).into_boxed_str()))?;

        let entry = |key| {
            localized
                .pointer(key)
                .and_then(|val| val.as_str())
                .map(String::from)
                .ok_or_else(|| err::Error::UnknownKey(String::from(key).into_boxed_str()))
        };

        let value = match entry(key) {
            Ok(value) => value,
            Err(e) => match opts.default_key {
                Some(ref default_key) => {
                    return entry(default_key);
                }
                _ => {
                    return Err(e);
                }
            },
        };

        match opts.vars {
            Some(vars) => {
                Ok(strfmt::strfmt(&value, &vars)?)
            }
            None => Ok(value),
        }
    }
}

/// Configuration to build a `Dictionary`
#[derive(Default, Clone)]
pub struct Config {
    load_paths: Vec<(Option<String>, PathBuf)>,
    load_path_pattern: Option<String>,
    default_locale: Option<String>,
}

impl Config {
    fn global() -> Self {
        Self {
            load_paths: Vec::new(),
            load_path_pattern: Some("config/locales/*.*".into()),
            default_locale: None,
        }
    }

    /// Add messages for a specific locale (e.g. `en`) from a specific file.
    pub fn with_localized_path<I: Into<String>, J: Into<PathBuf>>(
        mut self,
        locale: I,
        load_path: J,
    ) -> Self {
        self.load_paths.push((Some(locale.into()), load_path.into()));
        self
    }

    /// Use the specified glob pattern to add multiple files.
    ///
    /// Locale will be determined by the `file_stem`: e.g. `en.yml`. 
    pub fn with_path_pattern<I: Into<String>>(mut self, load_path_pattern: I) -> Self {
        self.load_path_pattern = Some(load_path_pattern.into());
        self
    }

    /// Set the default locale.
    pub fn with_default_locale<I: Into<String>>(mut self, default_locale: I) -> Self {
        self.default_locale = Some(default_locale.into());
        self
    }

    /// Build the `Dictionary` item.
    pub fn finish(mut self) -> err::Result<Dictionary> {
        let mut out = Dictionary::default();

        let glob_paths = match self.load_path_pattern {
            Some(load_path_pattern) => glob::glob(&load_path_pattern)
                .map_err(err::custom)?
                .flatten()
                .map(|x| (None, x))
                .collect::<Vec<_>>(),
            None => Vec::new(),
        };

        self.load_paths.extend(glob_paths);

        for (locale, path) in self.load_paths {
            let locale = locale
                .or_else(|| path.file_stem().map(|s| s.to_string_lossy().into_owned()))
                .ok_or_else(|| {
                    err::custom(format!(
                        "Couldn't determine `locale` for `path`: {}",
                        &path.display()
                    ))
                })?;

            let file = std::fs::File::open(&path)?;

            let value = match path.extension().and_then(|x| x.to_str()) {
                Some("yml") => serde_yaml::from_reader::<_, serde_json::Value>(&file)?,
                _ => {
                    continue;
                }
            };

            out.inner.insert(locale, value);
        }

        if let Some(locale) = self.default_locale {
            out.default_locale = locale;
        }

        Ok(out)
    }
}

static CONFIG: OnceCell<Config> = OnceCell::new();

/// Sets the `Config` to use for the global `translate` call.
pub fn set_config(config: Config) -> err::Result<()> {
    Ok(CONFIG.set(config.into()).map_err(|_| err::custom("`CONFIG` already set"))?)
}

/// Translate a message using the global configuration.
///
/// If you have not `set_config`, this will look for translation files in `config/locales`.
pub fn translate<'a, I: Into<Option<Opts<'a>>>>(key: &str, opts: I) -> err::Result<String> {
    let config = CONFIG.get_or_init(Config::global);

    let dict = Lazy::new(|| config.clone().finish().ok().unwrap_or_else(Dictionary::default));

    dict.translate(key, opts)
}

/// Shortcut for `translate`
pub fn t<'a, I: Into<Option<Opts<'a>>>>(key: &str, opts: I) -> err::Result<String> {
    translate(key, opts)
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn it_works() {
        set_config(Config::default().with_path_pattern("examples/locales/*.yml")).unwrap();

        assert_eq!(t("/greeting", None).unwrap(), String::from("Hello, World!"));

        assert_eq!(
            t("/missed", Opts::default().default_key("/missing/default")).unwrap(),
            String::from("Sorry, that translation doesn't exist.")
        );

        assert_eq!(
            t("/special-greeting", Opts::default().vars("name", "Jacob")).unwrap(),
            String::from("Hello, Jacob!!!")
        );

        assert_eq!(
            t("/greeting", Opts::default().locale("de")).unwrap(),
            String::from("Hallo Welt!")
        );

        assert_eq!(
            t("/messages", Opts::default().count(1)).unwrap(),
            String::from("You have one message.")
        );

        assert_eq!(
            t("/messages", Opts::default().count(0)).unwrap(),
            String::from("You have no messages.")
        );

        assert_eq!(
            t("/messages", Opts::default().count(200)).unwrap(),
            String::from("You have 200 messages.")
        );
    }
}
