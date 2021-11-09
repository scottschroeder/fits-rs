use crate::{
    fits::KEYWORD_LINE_LENGTH,
    parser::util::{exact_length, pair_values, ws},
    types::{CommentaryRecord, HeaderRecord, Keyword, KeywordRecord, Value},
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while, take_while1},
    character::is_digit,
    combinator::{map, map_res, not, opt, peek, recognize},
    multi::many0,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
use std::str::FromStr;

/// Use the `inner` parser to parse the next 80 bytes. Any final padding will be ignored.
fn parse_keyword_line<'a, F: 'a, O, E: nom::error::ParseError<&'a [u8]>>(
    inner: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>
where
    F: FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>,
    E: 'a,
    O: 'a,
{
    exact_length(KEYWORD_LINE_LENGTH, terminated(inner, many0(tag(" "))))
}

/// Parse a single record from a header
pub(crate) fn header_record(input: &[u8]) -> IResult<&[u8], HeaderRecord> {
    alt((keyword_record, end_record, blankfield_record))(input)
}

fn keyword_record(input: &[u8]) -> IResult<&[u8], HeaderRecord> {
    alt((commentary_keyword_record, value_keyword_record))(input)
}

fn commentary_keyword_record(input: &[u8]) -> IResult<&[u8], HeaderRecord> {
    // We are ignoring two possiblities here:
    // 1) blank records might have text?
    // 2) keyword=text
    // 3) Multi-line comments

    // TODO See 4.1.2.3
    parse_keyword_line(map(
        tuple((
            alt((comment_keyword, history_keyword)),
            opt(commenatry_text),
        )),
        |(keyword, commentary)| {
            HeaderRecord::CommentaryRecord(CommentaryRecord::new(keyword, commentary))
        },
    ))(input)
}

fn commenatry_text(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(take_while(is_ascii_text_char), |d| {
        std::str::from_utf8(d).map(|s| s.trim_end())
    })(input)
}

fn value_keyword_record(input: &[u8]) -> IResult<&[u8], HeaderRecord> {
    parse_keyword_line(map(
        tuple((keyword_field, value_indicator, ws(opt(value)), opt(comment))),
        |(keyword, _, value, comment)| {
            let value = value.unwrap_or(Value::Undefined);
            HeaderRecord::KeywordRecord(KeywordRecord::new(keyword, value, comment))
        },
    ))(input)
}

fn blankfield_record(input: &[u8]) -> IResult<&[u8], HeaderRecord> {
    parse_keyword_line(map(
        tuple((tag("        "), ws(opt(comment)))),
        |(_, comment)| HeaderRecord::BlankRecord(comment),
    ))(input)
}

fn end_record(input: &[u8]) -> IResult<&[u8], HeaderRecord> {
    parse_keyword_line(map(tag("END     "), |_| HeaderRecord::EndRecord))(input)
}

fn keyword_field(input: &[u8]) -> IResult<&[u8], Keyword> {
    map_res(
        map_res(take(8usize), std::str::from_utf8),
        Keyword::from_str,
    )(input)
}

fn comment_keyword(input: &[u8]) -> IResult<&[u8], Keyword> {
    map_res(
        map_res(tag("COMMENT "), std::str::from_utf8),
        Keyword::from_str,
    )(input)
}

fn history_keyword(input: &[u8]) -> IResult<&[u8], Keyword> {
    map_res(
        map_res(tag("HISTORY "), std::str::from_utf8),
        Keyword::from_str,
    )(input)
}

fn value_indicator(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag("= ")(input)
}

fn comment(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(
        preceded(tag("/"), ws(take_while(is_ascii_text_char))),
        |kw_bytes| std::str::from_utf8(kw_bytes).map(|s| s.trim_end()),
    )(input)
}

fn value(input: &[u8]) -> IResult<&[u8], Value> {
    alt((
        character_string_value,
        logical_value,
        integer_value,
        floating_value,
        complex_integer_value,
        complex_floating_value,
    ))(input)
}

fn character_string_value(input: &[u8]) -> IResult<&[u8], Value> {
    // Constraint: the begin_quote and end_quote are not part of the
    // character string value but only serve as delimiters. Leading
    // spaces are significant; trailing spaces are not.
    // TODO is a double-single-quote "''" an escaped version of "'"?
    map(
        map_res(
            delimited(tag("'"), take_while(is_string_text_char), tag("'")),
            std::str::from_utf8,
        ),
        |s| Value::CharacterString(s.trim_end()),
    )(input)
}

fn logical_value(input: &[u8]) -> IResult<&[u8], Value> {
    map(
        map_res(alt((tag("T"), tag("F"))), std::str::from_utf8),
        |s| {
            match s {
                "T" => Value::Logical(true),
                "F" => Value::Logical(false),
                _ => panic!("unknown value {:?}", s), // programmer error, should match alt block
            }
        },
    )(input)
}

fn integer_value(input: &[u8]) -> IResult<&[u8], Value> {
    map(integer, Value::Integer)(input)
}

// if an integer has a '.' after, we assume its a float and shouldn't parse
fn integer(input: &[u8]) -> IResult<&[u8], i64> {
    map_res(
        map_res(
            recognize(tuple((sign, take_while1(is_digit), peek(not(tag(".")))))),
            std::str::from_utf8,
        ),
        i64::from_str,
    )(input)
}

fn sign(input: &[u8]) -> IResult<&[u8], Option<u8>> {
    opt(map(alt((tag("+"), tag("-"))), |x: &[u8]| x[0]))(input)
}

fn floating_value(input: &[u8]) -> IResult<&[u8], Value> {
    map(floating, Value::Real)(input)
}

fn floating(input: &[u8]) -> IResult<&[u8], f64> {
    map_res(
        map_res(
            recognize(tuple((decimal_number, opt(exponent)))),
            std::str::from_utf8,
        ),
        f64::from_str, // TODO handle 3.14D2
    )(input)
}

fn decimal_number(input: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(tuple((
        opt(sign),
        alt((decimal_number_must_integer, decimal_number_must_fractional)),
    )))(input)
}

fn decimal_number_must_integer(input: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(tuple((
        number_part,
        opt(tuple((tag("."), opt(number_part)))),
    )))(input)
}

fn decimal_number_must_fractional(input: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(tuple((opt(number_part), tag("."), number_part)))(input)
}

fn number_part(input: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(is_digit)(input)
}

fn exponent(input: &[u8]) -> IResult<&[u8], &[u8]> {
    recognize(tuple((exponent_letter, opt(sign), number_part)))(input)
}
fn exponent_letter(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((tag("E"), tag("D")))(input)
}
fn complex_integer_value(input: &[u8]) -> IResult<&[u8], Value> {
    map(pair_values(integer, integer), |(r, c)| {
        Value::ComplexInteger((r, c))
    })(input)
}
fn complex_floating_value(input: &[u8]) -> IResult<&[u8], Value> {
    map(pair_values(floating, floating), |(r, c)| {
        Value::Complex((r, c))
    })(input)
}

fn is_ascii_text_char(chr: u8) -> bool {
    // Space - '~'
    (32u8..=126u8).contains(&chr)
}

fn is_string_text_char(chr: u8) -> bool {
    // TODO see 4.2.1: A single quote is represented
    // within a string as two successive single quotes, e.g., O’HARA =
    // ‘O’ ’HARA’. Leading spaces are significant; trailing spaces are
    // not.
    // Constraint: a string_text_char is identical to an ascii_text_char
    // except for the quote char; a quote char is represented by two
    // successive quote chars.
    let single_quote = b'\'';
    is_ascii_text_char(chr) && chr != single_quote
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyword_record_should_parse_a_keyword_record() {
        let data =
            "OBJECT  = 'EPIC 200164267'     / string version of target id                    "
                .as_bytes();
        assert_eq!(data.len(), KEYWORD_LINE_LENGTH);

        let (_, record) = keyword_record(data).unwrap();
        assert_eq!(
            record,
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::OBJECT,
                Value::CharacterString("EPIC 200164267"),
                Option::Some("string version of target id")
            ))
        )
    }

    #[test]
    fn keyword_record_should_parse_unrecognized_keyword_record() {
        let data =
            "SCALE_U =     0.00116355283466 / Upper-bound index scale (radians).             "
                .as_bytes();
        assert_eq!(data.len(), KEYWORD_LINE_LENGTH);

        let (_, record) = value_keyword_record(data).unwrap();
        assert_eq!(
            record,
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::Unrecognized("SCALE_U".to_string()),
                Value::Real(0.00116355283466f64),
                Option::Some("Upper-bound index scale (radians).")
            ))
        )
    }

    #[test]
    fn keyword_record_should_parse_a_keyword_record_without_a_comment() {
        let data =
            "KEPLERID=            200164267                                                  "
                .as_bytes();

        let (_, result) = keyword_record(data).unwrap();

        assert_eq!(
            result,
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::KEPLERID,
                Value::Integer(200164267),
                Option::None,
            ))
        )
    }

    #[test]
    fn header_record_should_parse_an_empty_comment() {
        let data =
            "               / string version of target id                                    "
                .as_bytes();
        assert_eq!(data.len(), KEYWORD_LINE_LENGTH);

        let (_, record) = header_record(data).unwrap();
        assert_eq!(
            record,
            HeaderRecord::BlankRecord(Option::Some("string version of target id"))
        )
    }

    #[allow(non_snake_case)]
    #[test]
    fn logical_constant_should_parse_an_uppercase_T_or_F() {
        for (constant, boolean) in &[("T", true), ("F", false)] {
            let data = constant.as_bytes();

            let (_, result) = logical_value(data).unwrap();
            assert_eq!(result, Value::Logical(*boolean))
        }
    }

    #[test]
    fn real_should_parse_an_floating_point_number() {
        for (input, f) in &[("1.0", 1f64), ("37.0", 37f64), ("51.0", 51f64)] {
            let data = input.as_bytes();

            let (_, result) = value(data).unwrap();
            assert_eq!(result, Value::Real(*f))
        }
    }

    #[test]
    fn integer_should_parse_an_integer() {
        for (input, n) in &[("1", 1i64), ("37", 37i64), ("51", 51i64)] {
            let data = input.as_bytes();

            let (_, result) = value(data).unwrap();
            assert_eq!(result, Value::Integer(*n))
        }
    }

    #[test]
    fn parse_character_string_value() {
        let data = "'EPIC 200164267'".as_bytes();
        let (_, k) = character_string_value(data).unwrap();
        assert_eq!(k, Value::CharacterString("EPIC 200164267"))
    }

    #[test]
    #[allow(clippy::float_cmp)] // we are testing parsing not math
    fn parse_float() {
        let data = "0.00116355283466".as_bytes();
        let (_, k) = floating(data).unwrap();
        assert_eq!(k, 0.00116355283466f64)
    }

    #[test]
    fn parse_real_value() {
        let data = "0.00116355283466".as_bytes();
        let (_, k) = value(data).unwrap();
        assert_eq!(k, Value::Real(0.00116355283466f64))
    }

    #[test]
    fn parse_integer_value() {
        let data = "8".as_bytes();
        let (_, k) = value(data).unwrap();
        assert_eq!(k, Value::Integer(8))
    }

    #[test]
    fn parse_single_keywords() {
        let data = "OBJECT  ".as_bytes();
        let (_, k) = keyword_field(data).unwrap();
        assert_eq!(k, Keyword::OBJECT)
    }

    #[test]
    fn parse_unrecognized_keywords() {
        let data = "SCALE_U ".as_bytes();
        let (_, k) = keyword_field(data).unwrap();
        assert_eq!(k, Keyword::Unrecognized("SCALE_U".to_string()))
    }

    #[test]
    fn parse_comment() {
        let data = "/ string version of target id".as_bytes();
        let (_, k) = comment(data).unwrap();
        assert_eq!(k, "string version of target id")
    }
}
