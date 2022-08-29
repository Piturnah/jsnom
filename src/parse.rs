use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take},
    character::complete::{char, digit0, digit1, multispace0, none_of, one_of},
    combinator::{map, opt, value},
    error::{ParseError, VerboseError},
    multi::{many0, separated_list0},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

use crate::JsonValue;

// whitespace delimited combinator from nom docs
fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

pub(crate) fn nom_parse(s: &str) -> IResult<&str, JsonValue, VerboseError<&str>> {
    alt((
        nom_null, nom_bool, nom_string, nom_array, nom_number, nom_object,
    ))(s)
}

pub(crate) fn nom_null(s: &str) -> IResult<&str, JsonValue, VerboseError<&str>> {
    map(ws(tag("null")), |_| JsonValue::Null)(s)
}

pub(crate) fn nom_bool(s: &str) -> IResult<&str, JsonValue, VerboseError<&str>> {
    match alt((ws(tag("true")), ws(tag("false"))))(s) {
        Ok((rest, "true")) => Ok((rest, JsonValue::Bool(true))),
        Ok((rest, "false")) => Ok((rest, JsonValue::Bool(false))),
        Err(e) => Err(e),
        _ => unreachable!(),
    }
}

fn nom_escaped_char(s: &str) -> IResult<&str, char, VerboseError<&str>> {
    preceded(
        char('\\'),
        alt((
            value('\"', char('"')),
            value('\\', char('\\')),
            value('\u{0008}', char('b')),
            value('\u{000c}', char('f')),
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\t', char('t')),
            // unicode literals
            map(tuple((char('u'), take(4usize))), |(_, code)| {
                char::from_u32(u32::from_str_radix(code, 16).unwrap()).unwrap()
            }),
        )),
    )(s)
}

pub(crate) fn nom_string(s: &str) -> IResult<&str, JsonValue, VerboseError<&str>> {
    match delimited(
        preceded(multispace0, char('"')),
        map(many0(alt((nom_escaped_char, none_of("\"")))), |cs| {
            cs.iter().collect::<String>()
        }),
        terminated(char('"'), multispace0),
    )(s)
    {
        Ok((rest, string)) => Ok((rest, JsonValue::String(string))),
        Err(e) => Err(e),
    }
}

pub(crate) fn nom_array(s: &str) -> IResult<&str, JsonValue, VerboseError<&str>> {
    map(
        delimited(
            ws(char('[')),
            terminated(separated_list0(char(','), nom_parse), opt(char(','))),
            ws(char(']')),
        ),
        JsonValue::Array,
    )(s)
}

pub(crate) fn nom_number(s: &str) -> IResult<&str, JsonValue, VerboseError<&str>> {
    // The JSON spec for numbers is pretty weird. You can have one leading 0 and then any number of
    // digits. Second digit in the `integer` part cannot be a 0. Also, +/- sign is ok for exponent
    // part, but the integer part can only have `-` or no sign.
    let integer = tuple((
        opt(tag("-")),
        one_of("1234567890"),
        opt(tuple((one_of("123456789"), digit0))),
    ));
    let floating = preceded(char('.'), digit0);
    let exponent = preceded(
        tag_no_case("e"),
        tuple((alt((char('+'), char('-'))), digit1)),
    );

    let (rest, ((minus, first, other), floating, exponent)) = delimited(
        multispace0,
        tuple((integer, opt(floating), opt(exponent))),
        multispace0,
    )(s)?;

    let mut number = minus.unwrap_or("").to_string() + &first.to_string();
    if let Some((first, second)) = other {
        number += &(first.to_string() + second);
    }

    if let Some(digits) = floating {
        number += ".";
        number += digits;
    }

    let mut number: f32 = number.parse().unwrap();

    if let Some((sign, exponent)) = exponent {
        let exponent = (sign.to_string() + exponent).parse().unwrap();
        number *= 10f32.powf(exponent);
    }

    Ok((rest, JsonValue::Number(number)))
}

pub(crate) fn nom_object(s: &str) -> IResult<&str, JsonValue, VerboseError<&str>> {
    let inner = terminated(
        separated_list0(
            char(','),
            pair(terminated(nom_string, char(':')), nom_parse),
        ),
        opt(char(',')),
    );
    let inner = delimited(ws(char('{')), inner, ws(char('}')));
    map(inner, JsonValue::Object)(s)
}

#[cfg(test)]
mod test {
    use super::JsonValue;

    #[test]
    fn nom_null() {
        assert_eq!(super::nom_null("null"), Ok(("", JsonValue::Null)));
    }

    #[test]
    fn nom_true() {
        assert_eq!(super::nom_bool("true"), Ok(("", JsonValue::Bool(true))));
    }

    #[test]
    fn nom_false() {
        assert_eq!(super::nom_bool("false"), Ok(("", JsonValue::Bool(false))));
    }

    #[test]
    fn nom_false_ws() {
        assert_eq!(super::nom_bool(" false "), Ok(("", JsonValue::Bool(false))));
    }

    #[test]
    fn nom_string() {
        assert_eq!(
            super::nom_string("\"hello, world!\""),
            Ok(("", JsonValue::String("hello, world!".to_string())))
        );
    }

    #[test]
    fn nom_char_escaped() {
        assert_eq!(super::nom_escaped_char("\\n"), Ok(("", '\n')))
    }

    #[test]
    fn nom_unicode() {
        assert_eq!(super::nom_escaped_char("\\u0d9e"), Ok(("", '\u{0d9e}')))
    }

    #[test]
    fn nom_string_escaped() {
        assert_eq!(
            super::nom_string("\"hello, world!\\n\""),
            Ok(("", JsonValue::String("hello, world!\n".to_string())))
        );
    }

    #[test]
    fn nom_array() {
        use JsonValue::*;
        assert_eq!(
            super::nom_array("[null, null , true,  false]"),
            Ok((
                "",
                JsonValue::Array(vec![Null, Null, Bool(true), Bool(false)])
            ))
        );
    }

    #[test]
    fn nom_array_nested() {
        use JsonValue::*;
        assert_eq!(
            super::nom_array("[[null, null] , true,  false]"),
            Ok((
                "",
                JsonValue::Array(vec![Array(vec![Null, Null]), Bool(true), Bool(false)])
            ))
        );
    }

    #[test]
    fn nom_integer() {
        assert_eq!(
            super::nom_number("0234"),
            Ok(("", JsonValue::Number(234.0)))
        );
    }

    #[test]
    #[should_panic]
    fn nom_bad_integer() {
        assert_eq!(
            super::nom_number("00234"),
            Ok(("", JsonValue::Number(234.0)))
        );
    }

    #[test]
    fn nom_float() {
        assert_eq!(
            super::nom_number("234.0123"),
            Ok(("", JsonValue::Number(234.0123)))
        );
    }

    #[test]
    fn nom_float_negative() {
        assert_eq!(
            super::nom_number("-234.0123"),
            Ok(("", JsonValue::Number(-234.0123)))
        );
    }

    #[test]
    fn nom_exponent() {
        assert_eq!(super::nom_number("3e-2"), Ok(("", JsonValue::Number(0.03))));
    }

    #[test]
    fn nom_object() {
        use super::JsonValue::*;
        assert_eq!(
            super::nom_object(
                "{\"item1\":null, \"item2\": null , \"my num\":  45, \"my_list\": [] }"
            ),
            Ok((
                "",
                JsonValue::Object(vec![
                    (String("item1".to_string()), Null),
                    (String("item2".to_string()), Null),
                    (String("my num".to_string()), Number(45.0)),
                    (String("my_list".to_string()), Array(Vec::new()))
                ])
            ))
        );
    }

    #[test]
    fn nom_object_single() {
        use super::JsonValue::*;
        assert_eq!(
            super::nom_object("{\"item1\":null }"),
            Ok((
                "",
                JsonValue::Object(vec![(String("item1".to_string()), Null),])
            ))
        );
    }

    #[test]
    fn nom_object_nested() {
        use super::JsonValue::*;
        assert_eq!(
            super::nom_object(
                "{\"item1\":null, \"item2\": null , \"my num\":  45, \"my_obj\": {} }"
            ),
            Ok((
                "",
                JsonValue::Object(vec![
                    (String("item1".to_string()), Null),
                    (String("item2".to_string()), Null),
                    (String("my num".to_string()), Number(45.0)),
                    (String("my_obj".to_string()), Object(Vec::new()))
                ])
            ))
        );
    }
}
