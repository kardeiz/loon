#[doc(hidden)]
#[derive(Clone)]
pub enum Key<'a> {
    Str(&'a str),
    Slice(&'a [&'a str]),
    Pair(Box<Key<'a>>, Box<Key<'a>>),
}

impl<'a> Key<'a> {
    fn dig<I: Iterator<Item = &'a str>>(
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

    fn iter(&self) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        match self {
            Key::Str(s) => Box::new(s.split('.')),
            Key::Slice(s) => Box::new(s.into_iter().map(|x| *x)),
            Key::Pair(a, b) => Box::new(a.iter().chain(b.iter())),
        }
    }

    pub(crate) fn chain<I: Into<Self>>(self, other: I) -> Self {
        Key::Pair(Box::new(self), Box::new(other.into()))
    }

    pub(crate) fn find(&'a self, value: &'a serde_json::Value) -> Option<&'a serde_json::Value> {
        Self::dig(self.iter(), value)
    }

    pub(crate) fn to_string(&self) -> String {
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
