use super::{err, Dictionary};
use std::path::PathBuf;

/// Helper for setting `default_locale` configuration
pub struct DefaultLocale<T>(pub T);
/// Helper for setting `path_pattern` configuration
pub struct PathPattern<T>(pub T);
/// Helper for setting `localized_path` configuration
pub struct LocalizedPath<T, U>(pub T, pub U);

pub trait ConfigPart {
    fn add_to(self, config: Config) -> Config;
}

impl<T> ConfigPart for DefaultLocale<T>
where
    T: Into<String>,
{
    fn add_to(self, config: Config) -> Config {
        config.with_default_locale(self.0)
    }
}

impl<T> ConfigPart for PathPattern<T>
where
    T: Into<String>,
{
    fn add_to(self, config: Config) -> Config {
        config.with_path_pattern(self.0)
    }
}

impl<T, U> ConfigPart for LocalizedPath<T, U>
where
    T: Into<String>,
    U: Into<PathBuf>,
{
    fn add_to(self, config: Config) -> Config {
        config.with_localized_path(self.0, self.1)
    }
}

impl<T> ConfigPart for (T,)
where
    T: ConfigPart,
{
    fn add_to(self, config: Config) -> Config {
        self.0.add_to(config)
    }
}

impl<'a, T, U> ConfigPart for (T, U)
where
    T: ConfigPart,
    U: ConfigPart,
{
    fn add_to(self, opts: Config) -> Config {
        self.1.add_to(self.0.add_to(opts))
    }
}

impl<'a, T, U, V> ConfigPart for (T, U, V)
where
    T: ConfigPart,
    U: ConfigPart,
    U: ConfigPart,
    V: ConfigPart,
{
    fn add_to(self, opts: Config) -> Config {
        self.2.add_to(self.1.add_to(self.0.add_to(opts)))
    }
}

impl<'a, T, U, V, W> ConfigPart for (T, U, V, W)
where
    T: ConfigPart,
    U: ConfigPart,
    U: ConfigPart,
    V: ConfigPart,
    W: ConfigPart,
{
    fn add_to(self, opts: Config) -> Config {
        self.3.add_to(self.2.add_to(self.1.add_to(self.0.add_to(opts))))
    }
}

impl<'a, T, U, V, W, X> ConfigPart for (T, U, V, W, X)
where
    T: ConfigPart,
    U: ConfigPart,
    U: ConfigPart,
    V: ConfigPart,
    W: ConfigPart,
    X: ConfigPart,
{
    fn add_to(self, opts: Config) -> Config {
        self.4.add_to(self.3.add_to(self.2.add_to(self.1.add_to(self.0.add_to(opts)))))
    }
}

impl<'a, T, U, V, W, X, Y> ConfigPart for (T, U, V, W, X, Y)
where
    T: ConfigPart,
    U: ConfigPart,
    U: ConfigPart,
    V: ConfigPart,
    W: ConfigPart,
    X: ConfigPart,
    Y: ConfigPart,
{
    fn add_to(self, opts: Config) -> Config {
        self.5
            .add_to(self.4.add_to(self.3.add_to(self.2.add_to(self.1.add_to(self.0.add_to(opts))))))
    }
}

impl<T> From<T> for Config
where
    T: ConfigPart,
{
    fn from(t: T) -> Self {
        t.add_to(Self::default())
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
    pub(crate) fn global() -> Self {
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
    /// Locale will be determined by the `file_stem`: e.g. `en.yml` for `locale` = `en`.
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
                Some("json") => serde_json::from_reader::<_, serde_json::Value>(&file)?,
                #[cfg(feature = "yaml")]
                Some("yml") => serde_yaml::from_reader::<_, serde_json::Value>(&file)?,
                #[cfg(feature = "toml")]
                Some("toml") => {
                    let mut file = file;
                    let mut buffer = Vec::new();
                    std::io::Read::read_to_end(&mut file, &mut buffer)?;
                    toml::from_slice::<serde_json::Value>(&buffer)?
                }
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
