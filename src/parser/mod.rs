//! The parser module is responsible for parsing FITS files.
//! This parser was created following the FITS 3.0 standard.
//! Specifically https://www.aanda.org/articles/aa/pdf/2010/16/aa15362-10.pdf
//! using Appendix A.
//!
//! We deviate from their organizational structure to make header END and <blank>
//! records easier to reason about.
use crate::{
    fits::KEYWORD_LINE_LENGTH,
    types::{CommentaryRecord, Keyword, KeywordRecord, Value, ValueRecord},
};
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while, take_while1},
    character::{complete::multispace0, is_digit},
    combinator::{eof, map, map_res, not, opt, peek, recognize, success},
    multi::{length_value, many0},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};
use std::str::FromStr;

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
    fn header_should_parse_a_primary_header() {
        let data =
            include_bytes!("../../assets/images/k2-trappist1-unofficial-tpf-long-cadence.fits");
        let (_, result) = header(&data[0..(10 * 2880)]).unwrap();
        let expect = long_cadence_header();
        for (idx, (r, e)) in result.into_iter().zip(expect).enumerate() {
            assert_eq!(r, e, "{}", idx)
        }
        // assert_eq!(result[start..end], long_cadence_header()[start..end])
    }

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
    fn keyword_record_should_parse_a_keyword_record_without_a_comment() {
        let data =
            "KEPLERID=            200164267                                                  "
                .as_bytes();

        let (_, result) = keyword_record(data).unwrap();

        assert_eq!(
            result,
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::KEPLERID,
                Value::Integer(200164267),
                Option::None,
            ))
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

    fn long_cadence_header<'a>() -> Vec<KeywordRecord<'a>> {
        vec![
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::SIMPLE,
                Value::Logical(true),
                Option::Some("conforms to FITS standards"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::BITPIX,
                Value::Integer(8i64),
                Option::Some("array data type"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::NAXIS,
                Value::Integer(0i64),
                Option::Some("number of array dimensions"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::EXTEND,
                Value::Logical(true),
                Option::Some("file contains extensions"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::NEXTEND,
                Value::Integer(2i64),
                Option::Some("number of standard extensions"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::EXTNAME,
                Value::CharacterString("PRIMARY"),
                Option::Some("name of extension"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::EXTVER,
                Value::Integer(1i64),
                Option::Some("extension version number (not format version)"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::ORIGIN,
                Value::CharacterString("Unofficial data product"),
                Option::Some("institution responsible for creating this"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::DATE,
                Value::CharacterString("2017-03-08"),
                Option::Some("file creation date."),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::CREATOR,
                Value::CharacterString("kadenza"),
                Option::Some("pipeline job and program u"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::PROCVER,
                Value::CharacterString("2.1.dev"),
                Option::Some("SW version"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::FILEVER,
                Value::CharacterString("0.0"),
                Option::Some("file format version"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::TIMVERSN,
                Value::CharacterString(""),
                Option::Some("OGIP memo number for file format"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::TELESCOP,
                Value::CharacterString("Kepler"),
                Option::Some("telescope"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::INSTRUME,
                Value::CharacterString("Kepler Photometer"),
                Option::Some("detector type"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::OBJECT,
                Value::CharacterString("EPIC 200164267"),
                Option::Some("string version of target id"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::KEPLERID,
                Value::Integer(200164267i64),
                Option::Some("unique Kepler target identifier"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::CHANNEL,
                Value::Integer(68i64),
                Option::Some("CCD channel"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::MODULE,
                Value::Integer(19i64),
                Option::Some("CCD module"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::OUTPUT,
                Value::Integer(4i64),
                Option::Some("CCD output"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::CAMPAIGN,
                Value::CharacterString(""),
                Option::Some("Observing campaign number"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::DATA_REL,
                Value::CharacterString(""),
                Option::Some("data release version number"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::OBSMODE,
                Value::CharacterString("long cadence"),
                Option::Some("observing mode"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::MISSION,
                Value::CharacterString("K2"),
                Option::Some("Mission name"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::TTABLEID,
                Value::CharacterString(""),
                Option::Some("target table id"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::RADESYS,
                Value::CharacterString("ICRS"),
                Option::Some("reference frame of celestial coordinates"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::RA_OBJ,
                Value::CharacterString(""),
                Option::Some("[deg] right ascension"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::DEC_OBJ,
                Value::CharacterString(""),
                Option::Some("[deg] declination"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::EQUINOX,
                Value::Real(2000.0f64),
                Option::Some("equinox of celestial coordinate system"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::PMRA,
                Value::Undefined,
                Option::Some("[arcsec/yr] RA proper motion"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::PMDEC,
                Value::Undefined,
                Option::Some("[arcsec/yr] Dec proper motion"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::PMTOTAL,
                Value::Undefined,
                Option::Some("[arcsec/yr] total proper motion"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::PARALLAX,
                Value::Undefined,
                Option::Some("[arcsec] parallax"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::GLON,
                Value::Undefined,
                Option::Some("[deg] galactic longitude"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::GLAT,
                Value::Undefined,
                Option::Some("[deg] galactic latitude"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::GMAG,
                Value::Undefined,
                Option::Some("[mag] SDSS g band magnitude"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::RMAG,
                Value::Undefined,
                Option::Some("[mag] SDSS r band magnitude"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::IMAG,
                Value::Undefined,
                Option::Some("[mag] SDSS i band magnitude"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::ZMAG,
                Value::Undefined,
                Option::Some("[mag] SDSS z band magnitude"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::JMAG,
                Value::Undefined,
                Option::Some("[mag] J band magnitude from 2MASS"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::HMAG,
                Value::Undefined,
                Option::Some("[mag] H band magnitude from 2MASS"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::KMAG,
                Value::Undefined,
                Option::Some("[mag] K band magnitude from 2MASS"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::KEPMAG,
                Value::Undefined,
                Option::Some("[mag] Kepler magnitude (Kp)"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::GRCOLOR,
                Value::Undefined,
                Option::Some("[mag] (g-r) color, SDSS bands"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::JKCOLOR,
                Value::Undefined,
                Option::Some("[mag] (J-K) color, 2MASS bands"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::GKCOLOR,
                Value::Undefined,
                Option::Some("[mag] (g-K) color, SDSS g - 2MASS K"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::TEFF,
                Value::Undefined,
                Option::Some("[K] Effective temperature"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::LOGG,
                Value::Undefined,
                Option::Some("[cm/s2] log10 surface gravity"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::FEH,
                Value::Undefined,
                Option::Some("[log10([Fe/H])]  metallicity"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::EBMINUSV,
                Value::Undefined,
                Option::Some("[mag] E(B-V) reddening"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::AV,
                Value::Undefined,
                Option::Some("[mag] A_v extinction"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::RADIUS,
                Value::Undefined,
                Option::Some("[solar radii] stellar radius"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::TMINDEX,
                Value::Undefined,
                Option::Some("unique 2MASS catalog ID"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::CHECKSUM,
                Value::CharacterString("7k7A7h637h697h69"),
                Option::Some("HDU checksum updated 2017-03-08T02:47:56"),
            )),
            KeywordRecord::ValueRecord(ValueRecord::new(
                Keyword::DATASUM,
                Value::CharacterString("0"),
                Option::Some("data unit checksum updated 2017-03-08T02:47:56"),
            )),
            KeywordRecord::EndRecord,
            KeywordRecord::BlankRecord,
            KeywordRecord::BlankRecord,
            KeywordRecord::BlankRecord,
            KeywordRecord::BlankRecord,
            KeywordRecord::BlankRecord,
            KeywordRecord::BlankRecord,
            KeywordRecord::BlankRecord,
            KeywordRecord::BlankRecord,
            KeywordRecord::BlankRecord,
            KeywordRecord::BlankRecord,
            KeywordRecord::BlankRecord,
            KeywordRecord::BlankRecord,
            KeywordRecord::BlankRecord,
            KeywordRecord::BlankRecord,
            KeywordRecord::BlankRecord,
        ]
    }
}
