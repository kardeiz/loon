use std::collections::HashMap;

use super::Key;

/// Helper for setting `locale` option
pub struct Locale<'a>(pub &'a str);
/// Helper for setting `default_key` option
pub struct DefaultKey<T>(pub T);
/// Helper for setting interpolated variables
pub struct Var<T, U>(pub T, pub U);
/// Helper for setting `count` option
pub struct Count(pub i32);

pub trait OptsPart<'a> {
    fn add_to(self, opts: Opts<'a>) -> Opts<'a>;
}

impl<'a> OptsPart<'a> for Locale<'a> {
    fn add_to(self, opts: Opts<'a>) -> Opts<'a> {
        opts.locale(self.0)
    }
}

impl<'a, T> OptsPart<'a> for DefaultKey<T>
where
    T: Into<Key<'a>>,
{
    fn add_to(self, opts: Opts<'a>) -> Opts<'a> {
        opts.default_key(self.0.into())
    }
}

impl<'a, T, U> OptsPart<'a> for Var<T, U>
where
    T: Into<String>,
    U: std::fmt::Display,
{
    fn add_to(self, opts: Opts<'a>) -> Opts<'a> {
        opts.var(self.0, self.1)
    }
}

impl<'a> OptsPart<'a> for Count {
    fn add_to(self, opts: Opts<'a>) -> Opts<'a> {
        opts.count(self.0)
    }
}

impl<'a, T> From<T> for Opts<'a>
where
    T: OptsPart<'a>,
{
    fn from(t: T) -> Self {
        t.add_to(Opts::default())
    }
}

impl<'a, T> OptsPart<'a> for (T,)
where
    T: OptsPart<'a>,
{
    fn add_to(self, opts: Opts<'a>) -> Opts<'a> {
        self.0.add_to(opts)
    }
}

impl<'a, T, U> OptsPart<'a> for (T, U)
where
    T: OptsPart<'a>,
    U: OptsPart<'a>,
{
    fn add_to(self, opts: Opts<'a>) -> Opts<'a> {
        self.1.add_to(self.0.add_to(opts))
    }
}

impl<'a, T, U, V> OptsPart<'a> for (T, U, V)
where
    T: OptsPart<'a>,
    U: OptsPart<'a>,
    U: OptsPart<'a>,
    V: OptsPart<'a>,
{
    fn add_to(self, opts: Opts<'a>) -> Opts<'a> {
        self.2.add_to(self.1.add_to(self.0.add_to(opts)))
    }
}

impl<'a, T, U, V, W> OptsPart<'a> for (T, U, V, W)
where
    T: OptsPart<'a>,
    U: OptsPart<'a>,
    U: OptsPart<'a>,
    V: OptsPart<'a>,
    W: OptsPart<'a>,
{
    fn add_to(self, opts: Opts<'a>) -> Opts<'a> {
        self.3.add_to(self.2.add_to(self.1.add_to(self.0.add_to(opts))))
    }
}

/// Options for the `translate` call
#[derive(Default)]
pub struct Opts<'a> {
    pub(crate) default_key: Option<Key<'a>>,
    pub(crate) vars: Option<HashMap<String, String>>,
    pub(crate) locale: Option<&'a str>,
    pub(crate) count: Option<i32>,
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

    /// Set any variables to be interpolated.
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
