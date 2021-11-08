//! The parser module is responsible for parsing FITS files.
//! This parser was created following the FITS 3.0 standard.
//! Specifically https://www.aanda.org/articles/aa/pdf/2010/16/aa15362-10.pdf
//! using Appendix A.
//!
//! We deviate from their organizational structure to make header END and <blank>
//! records easier to reason about.
use crate::types::CommentaryRecord;
use crate::types::Keyword;
use crate::types::KeywordRecord;
use crate::types::Value;
use crate::types::ValueRecord;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::bytes::complete::take_while;
use nom::bytes::complete::take_while1;
use nom::character::complete::multispace0;
use nom::character::is_digit;
use nom::combinator::eof;
use nom::combinator::opt;
use nom::combinator::recognize;
use nom::combinator::success;
use nom::error::dbg_dmp;
use nom::multi::length_value;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::sequence::terminated;
use nom::sequence::tuple;
use nom::{
    combinator::{map, map_res},
    IResult,
};
use std::str::FromStr;

/// All Keyword/Value/Comment lines are this fixed length
const KEYWORD_LINE_LENGTH: usize = 80;

/// All segments are in mulitples of this many bytes
const FITS_CHUNK_SIZE: usize = 36 * KEYWORD_LINE_LENGTH; // 2880

/// Parse the header data out of a FITS bytestream
pub fn parse_header(input: &[u8]) -> IResult<&[u8], Vec<KeywordRecord>> {
    // many0(keyword_record)(input)
    header(input)
}

/// A combinator that takes a length, and parser `inner` and produces a parser that
/// consumes the next length bytes, and returns the output of `inner`.
fn exact_length<'a, F: 'a, O, E: nom::error::ParseError<&'a [u8]>>(
    len: usize,
    inner: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>
where
    F: FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>,
{
    length_value(success(len), terminated(inner, eof))
}

/// A combinator that takes a parser `inner` and produces a parser that also consumes both leading and
/// trailing whitespace, returning the output of `inner`.
fn ws<'a, F: 'a, O, E: nom::error::ParseError<&'a [u8]>>(
    inner: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>
where
    F: FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>,
{
    delimited(multispace0, inner, multispace0)
}

/// Parse the results of two sub-parsers contained in a structure like ( A , B )
fn pair_values<'a, F: 'a, G: 'a, O1, O2, E: nom::error::ParseError<&'a [u8]>>(
    first: F,
    second: G,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], (O1, O2), E>
where
    F: FnMut(&'a [u8]) -> IResult<&'a [u8], O1, E>,
    G: FnMut(&'a [u8]) -> IResult<&'a [u8], O2, E>,
{
    map(
        delimited(tag("("), tuple((ws(first), tag(","), ws(second))), tag(")")),
        |t| (t.0, t.2),
    )
}

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

fn header(input: &[u8]) -> IResult<&[u8], Vec<KeywordRecord>> {
    map(
        tuple((many0(keyword_record), end_record, many0(blankfield_record))),
        |(mut keywords, end, blanks)| {
            keywords.push(end);
            keywords.extend(blanks.into_iter());
            keywords
        },
    )(input)
}

fn keyword_record(input: &[u8]) -> IResult<&[u8], KeywordRecord> {
    alt((commentary_keyword_record, value_keyword_record))(input)
}

fn commentary_keyword_record(input: &[u8]) -> IResult<&[u8], KeywordRecord> {
    // We are ignoring two possiblities here:
    // 1) blank records might have text?
    // 2) keyword=text

    // TODO See 4.1.2.3
    parse_keyword_line(map(
        tuple((
            alt((comment_keyword, history_keyword)),
            opt(commenatry_text),
        )),
        |(keyword, commentary)| {
            KeywordRecord::CommentaryRecord(CommentaryRecord::new(keyword, commentary))
        },
    ))(input)
}

fn commenatry_text(input: &[u8]) -> IResult<&[u8], &str> {
    map_res(take_while(is_ascii_text_char), std::str::from_utf8)(input)
}

fn value_keyword_record(input: &[u8]) -> IResult<&[u8], KeywordRecord> {
    parse_keyword_line(map(
        tuple((keyword_field, value_indicator, ws(opt(value)), opt(comment))),
        |(keyword, _, value, comment)| {
            let value = value.unwrap_or(Value::Undefined);
            KeywordRecord::ValueRecord(ValueRecord::new(keyword, value, comment))
        },
    ))(input)
}

fn blankfield_record(input: &[u8]) -> IResult<&[u8], KeywordRecord> {
    parse_keyword_line(map(tag("        "), |_| KeywordRecord::BlankRecord))(input)
}

fn end_record(input: &[u8]) -> IResult<&[u8], KeywordRecord> {
    parse_keyword_line(map(tag("END     "), |_| KeywordRecord::EndRecord))(input)
}

fn keyword_field(input: &[u8]) -> IResult<&[u8], Keyword> {
    map_res(
        map_res(take(8usize), std::str::from_utf8),
        Keyword::from_str,
    )(input)
}

fn comment_keyword(input: &[u8]) -> IResult<&[u8], Keyword> {
    map_res(
        map_res(tag("COMMENT "), |kw_bytes| std::str::from_utf8(kw_bytes)),
        Keyword::from_str,
    )(input)
}

fn history_keyword(input: &[u8]) -> IResult<&[u8], Keyword> {
    map_res(
        map_res(tag("HISTORY "), |kw_bytes| std::str::from_utf8(kw_bytes)),
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
        floating_value,
        integer_value,
        complex_floating_value,
        complex_integer_value,
    ))(input)
}

fn character_string_value(input: &[u8]) -> IResult<&[u8], Value> {
    // Constraint: the begin_quote and end_quote are not part of the
    // character string value but only serve as delimiters. Leading
    // spaces are significant; trailing spaces are not.
    // TODO should we trim?
    // TODO is a double-single-quote "''" an escaped version of "'"?
    map(
        map_res(
            delimited(tag("'"), take_while(is_string_text_char), tag("'")),
            std::str::from_utf8,
        ),
        Value::CharacterString,
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

fn integer(input: &[u8]) -> IResult<&[u8], i64> {
    map_res(
        map_res(
            recognize(tuple((sign, take_while1(is_digit)))),
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
// fn old_floating_value(input: &[u8]) -> IResult<&[u8], Value> {
//     map(
//         map_res(
//             map_res(
//                 recognize(tuple((
//                     take_while(is_digit),
//                     tag("."),
//                     take_while(is_digit),
//                 ))),
//                 std::str::from_utf8,
//             ),
//             f64::from_str,
//         ),
//         Value::Real,
//     )(input)
// }

fn is_ascii_text_char(chr: u8) -> bool {
    // Space - '~'
    32u8 <= chr && chr <= 126u8
}

fn is_anychar_but_equal(chr: u8) -> bool {
    let equal = '=' as u8;
    is_ascii_text_char(chr) && chr != equal
}

fn is_anychar_but_space(chr: u8) -> bool {
    let space = ' ' as u8;
    is_ascii_text_char(chr) && chr != space
}

fn is_string_text_char(chr: u8) -> bool {
    // Constraint: a string_text_char is identical to an ascii_text_char
    // except for the quote char; a quote char is represented by two
    // successive quote chars.

    // TODO see 4.2.1: A single quote is represented
    // within a string as two successive single quotes, e.g., O’HARA =
    // ‘O’ ’HARA’. Leading spaces are significant; trailing spaces are
    // not.
    let single_quote = '\'' as u8;
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
            KeywordRecord::ValueRecord(ValueRecord::new(
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
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::Unrecognized("SCALE_U".to_string()),
                Value::Real(0.00116355283466f64),
                Option::Some("Upper-bound index scale (radians).")
            ))
        )
    }

    #[test]
    fn parse_character_string_value() {
        let data = "'EPIC 200164267'".as_bytes();
        let (_, k) = character_string_value(data).unwrap();
        assert_eq!(k, Value::CharacterString("EPIC 200164267"))
    }

    #[test]
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
