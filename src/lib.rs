//! ## lo[calizati]on
//!
//! A very simple localization/internationalization provider, inspired by `ruby-i18n`.
//!
//! ## Usage:
//!
//! ```rust
//! fn main() {
//!
//!     use loon::{t, Var, Opts};
//!     
//!     loon::set_config(loon::Config::default().with_path_pattern("examples/locales/*.yml")).unwrap();
//!
//!     assert_eq!(
//!         t("custom.greeting", Var("name", "Jacob")).unwrap(),
//!         String::from("Hello, Jacob!!!")
//!     );
//!
//!     assert_eq!(
//!         t("greeting", Opts::default().locale("de")).unwrap(),
//!         String::from("Hallo Welt!")
//!     );
//! }
//! ```

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

mod utils {
    pub(crate) fn dig<'a, I: Iterator<Item = &'a str>>(
        path: I,
        mut value: &serde_json::Value,
    ) -> Option<&serde_json::Value> {
        for part in path {
            let value_opt = match value {
                serde_json::Value::Object(ref map) => map.get(part),
                serde_json::Value::Array(ref arr) => {
                    part.parse::<usize>().ok().and_then(|i| arr.get(i))
                }
                _ => None,
            };
            value = match value_opt {
                Some(value) => value,
                None => {
                    return None;
                }
            }
        }
        Some(value)
    }
}

use once_cell::sync::{Lazy, OnceCell};
use std::collections::HashMap;
use std::path::PathBuf;

#[doc(hidden)]
#[derive(Clone)]
pub enum Key<'a> {
    Str(&'a str),
    Slice(&'a [&'a str]),
    Pair(Box<Key<'a>>, Box<Key<'a>>),
}

impl<'a> Key<'a> {
    fn chain<I: Into<Self>>(self, other: I) -> Self {
        Key::Pair(Box::new(self), Box::new(other.into()))
    }

    fn iter(&self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        match self {
            Key::Str(s) => Box::new(s.split('.')),
            Key::Slice(s) => Box::new(s.into_iter().map(|x| *x)),
            Key::Pair(a, b) => Box::new(a.iter().chain(b.iter())),
        }
    }

    fn find(&'a self, value: &'a serde_json::Value) -> Option<&'a serde_json::Value> {
        utils::dig(self.iter(), value)
    }

    fn to_string(&self) -> String {
        self.iter().collect::<Vec<_>>().join(".")
    }
}

impl<'a> From<&'a str> for Key<'a> {
    fn from(t: &'a str) -> Self {
        Key::Str(t)
    }
}

impl<'a> From<&'a [&'a str]> for Key<'a> {
    fn from(t: &'a [&'a str]) -> Self {
        Key::Slice(t)
    }
}

impl<'a> From<&'a [&'a str; 1]> for Key<'a> {
    fn from(t: &'a [&'a str; 1]) -> Self {
        Key::Slice(t)
    }
}

impl<'a> From<&'a [&'a str; 2]> for Key<'a> {
    fn from(t: &'a [&'a str; 2]) -> Self {
        Key::Slice(t)
    }
}

impl<'a> From<&'a [&'a str; 3]> for Key<'a> {
    fn from(t: &'a [&'a str; 3]) -> Self {
        Key::Slice(t)
    }
}

impl<'a> From<&'a [&'a str; 4]> for Key<'a> {
    fn from(t: &'a [&'a str; 4]) -> Self {
        Key::Slice(t)
    }
}

impl<'a> From<&'a [&'a str; 5]> for Key<'a> {
    fn from(t: &'a [&'a str; 5]) -> Self {
        Key::Slice(t)
    }
}

impl<'a> From<&'a [&'a str; 6]> for Key<'a> {
    fn from(t: &'a [&'a str; 6]) -> Self {
        Key::Slice(t)
    }
}

impl<'a> From<&'a [&'a str; 7]> for Key<'a> {
    fn from(t: &'a [&'a str; 7]) -> Self {
        Key::Slice(t)
    }
}

impl<'a> From<&'a [&'a str; 8]> for Key<'a> {
    fn from(t: &'a [&'a str; 8]) -> Self {
        Key::Slice(t)
    }
}

impl<'a> From<&'a [&'a str; 9]> for Key<'a> {
    fn from(t: &'a [&'a str; 9]) -> Self {
        Key::Slice(t)
    }
}

impl<'a> From<&'a [&'a str; 10]> for Key<'a> {
    fn from(t: &'a [&'a str; 10]) -> Self {
        Key::Slice(t)
    }
}

impl<'a> From<&'a [&'a str; 11]> for Key<'a> {
    fn from(t: &'a [&'a str; 11]) -> Self {
        Key::Slice(t)
    }
}

impl<'a> From<&'a [&'a str; 12]> for Key<'a> {
    fn from(t: &'a [&'a str; 12]) -> Self {
        Key::Slice(t)
    }
}

/// Helper for setting `locale` option
pub struct Locale<'a>(pub &'a str);
/// Helper for setting `default_key` option
pub struct DefaultKey<T>(pub T);
/// Helper for setting interpolated variables
pub struct Var<T, U>(pub T, pub U);

/// Used for the alternate options form.
pub trait WithOpt<'a> {
    fn with_opt(self, opts: Opts<'a>) -> Opts<'a>;
}

impl<'a> WithOpt<'a> for Locale<'a> {
    fn with_opt(self, opts: Opts<'a>) -> Opts<'a> {
        opts.locale(self.0)
    }
}

impl<'a, T> WithOpt<'a> for DefaultKey<T>
where
    T: Into<Key<'a>>,
{
    fn with_opt(self, opts: Opts<'a>) -> Opts<'a> {
        opts.default_key(self.0.into())
    }
}

impl<'a, T, U> WithOpt<'a> for Var<T, U>
where
    T: Into<String>,
    U: std::fmt::Display,
{
    fn with_opt(self, opts: Opts<'a>) -> Opts<'a> {
        opts.var(self.0, self.1)
    }
}

impl<'a> WithOpt<'a> for i32 {
    fn with_opt(self, opts: Opts<'a>) -> Opts<'a> {
        opts.count(self)
    }
}

impl<'a, T> From<T> for Opts<'a>
where
    T: WithOpt<'a>,
{
    fn from(t: T) -> Self {
        t.with_opt(Opts::default())
    }
}

impl<'a, T> WithOpt<'a> for (T,)
where
    T: WithOpt<'a>,
{
    fn with_opt(self, opts: Opts<'a>) -> Opts<'a> {
        self.0.with_opt(opts)
    }
}

impl<'a, T, U> WithOpt<'a> for (T, U)
where
    T: WithOpt<'a>,
    U: WithOpt<'a>,
{
    fn with_opt(self, opts: Opts<'a>) -> Opts<'a> {
        self.1.with_opt(self.0.with_opt(opts))
    }
}

impl<'a, T, U, V> WithOpt<'a> for (T, U, V)
where
    T: WithOpt<'a>,
    U: WithOpt<'a>,
    U: WithOpt<'a>,
    V: WithOpt<'a>,
{
    fn with_opt(self, opts: Opts<'a>) -> Opts<'a> {
        self.2.with_opt(self.1.with_opt(self.0.with_opt(opts)))
    }
}

impl<'a, T, U, V, W> WithOpt<'a> for (T, U, V, W)
where
    T: WithOpt<'a>,
    U: WithOpt<'a>,
    U: WithOpt<'a>,
    V: WithOpt<'a>,
    W: WithOpt<'a>,
{
    fn with_opt(self, opts: Opts<'a>) -> Opts<'a> {
        self.3.with_opt(self.2.with_opt(self.1.with_opt(self.0.with_opt(opts))))
    }
}

/// Options (optional) for the `translate` call
#[derive(Default)]
pub struct Opts<'a> {
    default_key: Option<Key<'a>>,
    vars: Option<HashMap<String, String>>,
    locale: Option<&'a str>,
    count: Option<i32>,
}

impl<'a> Opts<'a> {
    /// If the key does not exist, fallback to using another key.
    pub fn default_key<I: Into<Key<'a>>>(mut self, default_key: I) -> Self {
        self.default_key = Some(default_key.into());
        self
    }

    /// Set the locale for this `translate` call.
    pub fn locale(mut self, locale: &'a str) -> Self {
        self.locale = Some(locale);
        self
    }

    /// Set any variables to be inerpolated.
    pub fn var<I: Into<String>, J: std::fmt::Display>(mut self, key: I, value: J) -> Self {
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
        self.var("count", count)
    }
}

impl<'a> From<Option<Opts<'a>>> for Opts<'a> {
    fn from(t: Option<Opts<'a>>) -> Self {
        t.unwrap_or_else(Opts::default)
    }
}

impl<'a> From<()> for Opts<'a> {
    fn from(_: ()) -> Self {
        Opts::default()
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
    ///
    /// `key` can be a dot-delimited `&str` or a `&[&str]` path.
    /// `opts` can be an `Opts` object, `None`, or an item that implements `WithOpt`.
    pub fn translate<'a, K: Into<Key<'a>>, I: Into<Opts<'a>>>(
        &self,
        key: K,
        opts: I,
    ) -> err::Result<String> {
        let opts = opts.into();

        let mut key = key.into();

        let alt_key;

        match opts.count {
            Some(0) => {
                alt_key = key.chain(["zero"].as_ref());
                key = alt_key;
            }
            Some(1) => {
                alt_key = key.chain(["one"].as_ref());
                key = alt_key;
            }
            Some(_) => {
                alt_key = key.chain(["other"].as_ref());
                key = alt_key;
            }
            _ => {}
        }

        let locale = opts.locale.unwrap_or_else(|| &self.default_locale);

        let localized = self
            .inner
            .get(locale)
            .ok_or_else(|| err::Error::UnknownLocale(String::from(locale).into_boxed_str()))?;

        let entry = |key: Key| {
            key.find(localized)
                .and_then(|val| val.as_str())
                .map(String::from)
                .ok_or_else(|| err::Error::UnknownKey(key.to_string().into_boxed_str()))
        };

        let value = match entry(key) {
            Ok(value) => value,
            Err(e) => match opts.default_key {
                Some(default_key) => {
                    return entry(default_key);
                }
                _ => {
                    return Err(e);
                }
            },
        };

        match opts.vars {
            Some(vars) => Ok(strfmt::strfmt(&value, &vars)?),
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
pub fn translate<'a, K: Into<Key<'a>>, I: Into<Opts<'a>>>(key: K, opts: I) -> err::Result<String> {
    static DICTIONARY: Lazy<Dictionary> = Lazy::new(|| {
        CONFIG.get_or_init(Config::global).clone().finish().ok().unwrap_or_else(Dictionary::default)
    });

    DICTIONARY.translate(key, opts)
}

/// Shortcut for `translate`
pub fn t<'a, K: Into<Key<'a>>, I: Into<Opts<'a>>>(key: K, opts: I) -> err::Result<String> {
    translate(key, opts)
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn it_works() {
        set_config(Config::default().with_path_pattern("examples/locales/*.yml")).unwrap();

        assert_eq!(t(&["greeting"], None).unwrap(), String::from("Hello, World!"));

        assert_eq!(
            t("missed", DefaultKey("missing.default")).unwrap(),
            String::from("Sorry, that translation doesn't exist.")
        );

        assert_eq!(
            t(&["custom", "greeting"], Var("name", "Jacob")).unwrap(),
            String::from("Hello, Jacob!!!")
        );

        assert_eq!(
            t("greeting", Opts::default().locale("de")).unwrap(),
            String::from("Hallo Welt!")
        );

        assert_eq!(
            t("messages", Opts::default().count(1)).unwrap(),
            String::from("You have one message.")
        );

        assert_eq!(
            t("messages", Opts::default().count(0)).unwrap(),
            String::from("You have no messages.")
        );

        assert_eq!(t("messages", 200).unwrap(), String::from("You have 200 messages."));

        assert_eq!(
            t(
                "a.very.nested.message",
                (Var("name", "you"), Var("message", "\"a very nested message\""))
            )
            .unwrap(),
            String::from("Hello, you. Your message is: \"a very nested message\"")
        );
    }
}
