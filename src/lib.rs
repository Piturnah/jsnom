//! # JSnom
//!
//! A small and ergonomic parser library for JSON.
//!
//! All parsers provided silently discard any of the string remaining after parser finishes.
//!
//! ## Example
//!
//! ```
//! use jsnom::JsonValue;
//!
//! assert_eq!(
//!     JsonValue::from_str("[null, null, true]"),
//!     Ok(JsonValue::Array(vec![
//!         JsonValue::Null,
//!         JsonValue::Null,
//!         JsonValue::Bool(true)
//!     ]))
//! )
//! ```

use std::fmt;

use nom::{
    error::{convert_error, VerboseError, VerboseErrorKind},
    Finish,
};

mod parse;

/// Enum representing a parsed JSON input.
#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    String(String),
    Array(Vec<JsonValue>),
    Number(f32),
    Object(Vec<(JsonValue, JsonValue)>),
}

/// The error type returned from parsers. It is essentially a wrapper around
/// [`nom::error::VerboseError`] using a different [`std::fmt::Display`].
#[derive(Clone, Debug, PartialEq)]
pub struct Error<'a> {
    pub errors: Vec<(&'a str, VerboseErrorKind)>,
    data: &'a str,
    raw_error: VerboseError<&'a str>,
}

impl<'a> std::error::Error for Error<'a> {}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", convert_error(self.data, self.raw_error.clone()))
    }
}

impl<'a> Error<'a> {
    fn from_raw(data: &'a str, raw: VerboseError<&'a str>) -> Self {
        Self {
            errors: raw.clone().errors,
            data,
            raw_error: raw,
        }
    }
}

impl JsonValue {
    /// Parse a [`JsonValue`] from an input string.
    ///
    /// ```
    /// use jsnom::JsonValue;
    ///
    /// assert_eq!(
    ///     JsonValue::from_str("[null, null, true]"),
    ///     Ok(JsonValue::Array(vec![
    ///         JsonValue::Null,
    ///         JsonValue::Null,
    ///         JsonValue::Bool(true)
    ///     ]))
    /// )
    /// ```
    #[allow(clippy::should_implement_trait)]
    // We cannot implement `FromStr` due to lifetimes
    pub fn from_str(s: &str) -> Result<Self, Error> {
        parse(s)
    }
}

/// Parse a [`JsonValue`] from an input string.
pub fn parse(s: &str) -> Result<JsonValue, Error> {
    parse::nom_parse(s)
        .finish()
        .map(|(_, val)| val)
        .map_err(|e| Error::from_raw(s, e))
}

/// Parse a [`JsonValue::Null`] from an input string.
///
/// ```
/// use jsnom::{parse_null, JsonValue};
///
/// assert_eq!(parse_null("null"), Ok(JsonValue::Null));
/// ```
pub fn parse_null(s: &str) -> Result<JsonValue, Error> {
    parse::nom_null(s)
        .finish()
        .map(|(_, val)| val)
        .map_err(|e| Error::from_raw(s, e))
}

/// Parse a [`JsonValue::Bool`] from an input string.
/// ```
/// use jsnom::{parse_bool, JsonValue};
///
/// assert_eq!(parse_bool("true"), Ok(JsonValue::Bool(true)));
/// ```
pub fn parse_bool(s: &str) -> Result<JsonValue, Error> {
    parse::nom_bool(s)
        .finish()
        .map(|(_, val)| val)
        .map_err(|e| Error::from_raw(s, e))
}

/// Parse a [`JsonValue::String`] from an input string.
/// ```
/// use jsnom::{parse_string, JsonValue};
///
/// assert_eq!(
///     parse_string("\"Hello, world!\\n\""),
///     Ok(JsonValue::String("Hello, world!\n".to_string()))
/// );
/// ```
pub fn parse_string(s: &str) -> Result<JsonValue, Error> {
    parse::nom_string(s)
        .finish()
        .map(|(_, val)| val)
        .map_err(|e| Error::from_raw(s, e))
}

/// Parse a [`JsonValue::Array`] from an input string.
/// ```
/// use jsnom::{parse_array, JsonValue};
///
/// assert_eq!(
///     parse_array("[null, null, [\"hello\", false]]"),
///     Ok(JsonValue::Array(vec![
///         JsonValue::Null,
///         JsonValue::Null,
///         JsonValue::Array(vec![
///             JsonValue::String("hello".to_string()),
///             JsonValue::Bool(false)
///         ])
///     ]))
/// );
/// ```
pub fn parse_array(s: &str) -> Result<JsonValue, Error> {
    parse::nom_array(s)
        .finish()
        .map(|(_, val)| val)
        .map_err(|e| Error::from_raw(s, e))
}

/// Parse a [`JsonValue::Number`] from an input string.
/// ```
/// use jsnom::{parse_number, JsonValue};
///
/// assert_eq!(parse_number("-3e-2"), Ok(JsonValue::Number(-0.03)));
/// ```
pub fn parse_number(s: &str) -> Result<JsonValue, Error> {
    parse::nom_number(s)
        .finish()
        .map(|(_, val)| val)
        .map_err(|e| Error::from_raw(s, e))
}

/// Parse a [`JsonValue::Object`] from an input string.
/// ```
/// use jsnom::{parse_object, JsonValue::{self, *}};
///
/// assert_eq!(
///     parse_object("{\"user\": \"Piturnah\", \"crates\": [\"gex\", \"newdoku\", \"jsnom\"]}"),
///     Ok(JsonValue::Object(vec![
///         (String("user".to_string()), String("Piturnah".to_string())),
///         (String("crates".to_string()), Array(vec![
///             String("gex".to_string()),
///             String("newdoku".to_string()),
///             String("jsnom".to_string()),
///         ]))
///     ])));
/// ```
pub fn parse_object(s: &str) -> Result<JsonValue, Error> {
    parse::nom_object(s)
        .finish()
        .map(|(_, val)| val)
        .map_err(|e| Error::from_raw(s, e))
}
