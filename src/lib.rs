//! <h2>lo<span style="color:Silver;">[calizati]</span>on</h2>
//!
//! A very simple localization/internationalization library, inspired by `ruby-i18n`.
//!
//! Provides a (configurable) global `translate`/`t` function for convenience, as well
//! as a `Dictionary` builder/container if you prefer to manage state directly.
//!
//! ## Usage:
//!
//! Global function:
//!
//! ```rust
//! fn main() {
//!
//!     use loon::prelude::*;
//!     
//!     loon::set_config(PathPattern("examples/locales/*.yml")).unwrap();
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
//!
//! Using a `Dictionary`:
//!
//! ```rust
//! fn main() {
//!
//!     use loon::prelude::*;
//!     
//!     let dict = Config::default()
//!         .with_path_pattern("examples/locales/*.yml")
//!         .finish()
//!         .unwrap();
//!
//!     assert_eq!(
//!         dict.translate("custom.greeting", Var("name", "Jacob")).unwrap(),
//!         String::from("Hello, Jacob!!!")
//!     );
//!
//!     assert_eq!(
//!         dict.translate("greeting", Opts::default().locale("de")).unwrap(),
//!         String::from("Hallo Welt!")
//!     );
//! }
//! ```
//!
//! ## Features
//!
//! Translation files can be:
//! * JSON
//! * YAML (enabled by default, disable with `default-features = false`), or
//! * TOML (enable with `features = ["toml"]`).

/// Error management
pub mod err {

    /// Error container
    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        #[error("IO error: `{0}`")]
        Io(#[from] std::io::Error),
        #[error("strfmt error: `{0}`")]
        Strfmt(#[from] strfmt::FmtError),
        #[cfg(feature = "yaml")]
        #[error("YAML error: {0}")]
        Yaml(#[from] serde_yaml::Error),
        #[error("JSON error: {0}")]
        Json(#[from] serde_json::Error),
        #[cfg(feature = "toml")]
        #[error("TOML error: {0}")]
        Toml(#[from] toml::de::Error),
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

mod config;
mod key;
mod opts;

/// Helpers to build `Config` or `Opts` items
pub mod helpers {

    /// Helpers to build `Config` items
    pub mod config {
        pub use crate::config::{DefaultLocale, LocalizedPath, PathPattern};
    }

    /// Helpers to build `Opts` items
    pub mod opts {
        pub use crate::opts::{Count, DefaultKey, Locale, Var};
    }
}

/// For convenience
///
/// ```rust
/// pub use loon::{
///     helpers::{config::*, opts::*},
///     Config,
///     Dictionary,
///     Opts,
///     translate,
///     t
/// };
/// ```
pub mod prelude {
    pub use crate::{
        helpers::{config::*, opts::*},
        Config,
        Dictionary,
        Opts,
        translate,
        t
    };
}

use once_cell::sync::{Lazy, OnceCell};
use std::collections::HashMap;

pub use config::Config;
pub use key::Key;
pub use opts::Opts;

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
    /// Get the translated message.
    ///
    /// `key` can be a dot-delimited `&str` or a `&[&str]` path.
    ///
    /// `opts` can be an `Opts` object, `None`, or `Var`, `Count`, `Locale`, or `DefaultKey` (or up
    /// to a `4-tuple` of these items).
    ///
    /// Examples:
    /// ```rust
    /// use loon::prelude::*;
    /// let dict = Dictionary::default();
    /// let _ = dict.translate("custom.greeting", Opts::default().var("name", "Jacob"));
    /// let _ = dict.translate(&["custom", "greeting"], Var("name", "Jacob"));    
    /// let _ = dict.translate("greeting", None);
    /// let _ = dict.translate("greeting", (Locale("de"), (DefaultKey("missing.message"))));
    /// ```
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

    /// Shortcut for `translate`.
    ///
    /// `key` can be a dot-delimited `&str` or a `&[&str]` path.
    ///
    /// `opts` can be an `Opts` object, `None`, or `Var`, `Count`, `Locale`, or `DefaultKey` (or up
    /// to a `4-tuple` of these items).
    ///
    /// Examples:
    /// ```rust
    /// use loon::prelude::*;
    /// let dict = Dictionary::default();
    /// let _ = dict.t("custom.greeting", Opts::default().var("name", "Jacob"));
    /// let _ = dict.t(&["custom", "greeting"], Var("name", "Jacob"));    
    /// let _ = dict.t("greeting", None);
    /// let _ = dict.t("greeting", (Locale("de"), (DefaultKey("missing.message"))));
    /// ```
    pub fn t<'a, K: Into<Key<'a>>, I: Into<Opts<'a>>>(
        &self,
        key: K,
        opts: I,
    ) -> err::Result<String> {
        self.translate(key, opts)
    }
}

static CONFIG: OnceCell<Config> = OnceCell::new();

/// Sets the `Config` to use for the global `translate` call.
///
/// `config` can be a `Config` object, or `DefaultLocale`, `PathPattern`, or `LocalizedPath` (or up
/// to a `4-tuple` of these items).
///
/// Examples:
/// ```rust
/// use loon::prelude::*;
/// loon::set_config(Config::default().with_path_pattern("examples/locales/*.yml"));
/// loon::set_config(PathPattern("examples/locales/*.yml"));
/// loon::set_config((PathPattern("examples/locales/*.yml"), DefaultLocale("en")));
/// ```
pub fn set_config<I: Into<Config>>(config: I) -> err::Result<()> {
    Ok(CONFIG.set(config.into()).map_err(|_| err::custom("`CONFIG` already set"))?)
}

/// Get the translated message, using the global configuration.
///
/// `key` can be a dot-delimited `&str` or a `&[&str]` path.
///
/// `opts` can be an `Opts` object, `None`, or `Var`, `Count`, `Locale`, or `DefaultKey` (or up
/// to a `4-tuple` of these items).
///
/// Examples:
/// ```rust
/// use loon::prelude::*;
/// let _ = translate("custom.greeting", Opts::default().var("name", "Jacob"));
/// let _ = translate(&["custom", "greeting"], Var("name", "Jacob"));    
/// let _ = translate("greeting", None);
/// let _ = translate("greeting", (Locale("de"), (DefaultKey("missing.message"))));
/// ```
pub fn translate<'a, K: Into<Key<'a>>, I: Into<Opts<'a>>>(key: K, opts: I) -> err::Result<String> {
    static DICTIONARY_RESULT: Lazy<err::Result<Dictionary>> =
        Lazy::new(|| CONFIG.get_or_init(Config::global).clone().finish());

    DICTIONARY_RESULT.as_ref().map_err(err::custom).and_then(|dict| dict.translate(key, opts))
}

/// Shortcut for `translate`.
///
/// `key` can be a dot-delimited `&str` or a `&[&str]` path.
///
/// `opts` can be an `Opts` object, `None`, or `Var`, `Count`, `Locale`, or `DefaultKey` (or up
/// to a `4-tuple` of these items).
///
/// Examples:
/// ```rust
/// use loon::prelude::*;
/// let _ = t("custom.greeting", Opts::default().var("name", "Jacob"));
/// let _ = t(&["custom", "greeting"], Var("name", "Jacob"));    
/// let _ = t("greeting", None);
/// let _ = t("greeting", (Locale("de"), (DefaultKey("missing.message"))));
/// ```
pub fn t<'a, K: Into<Key<'a>>, I: Into<Opts<'a>>>(key: K, opts: I) -> err::Result<String> {
    translate(key, opts)
}

#[cfg(test)]
mod tests {

    use crate::prelude::*;

    #[test]
    fn it_works() {
        crate::set_config(PathPattern("examples/locales/*.yml")).unwrap();

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

        assert_eq!(t("messages", Count(200)).unwrap(), String::from("You have 200 messages."));

        assert!(t("message.x", ()).is_err());

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
