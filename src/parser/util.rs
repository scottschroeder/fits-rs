use nom::{
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::{eof, map, success},
    multi::length_value,
    sequence::{delimited, terminated, tuple},
    IResult,
};

/// A combinator that takes a length, and parser `inner` and produces a parser that
/// consumes the next length bytes, and returns the output of `inner`.
pub(crate) fn exact_length<'a, F: 'a, O, E: nom::error::ParseError<&'a [u8]>>(
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
pub(crate) fn ws<'a, F: 'a, O, E: nom::error::ParseError<&'a [u8]>>(
    inner: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>
where
    F: FnMut(&'a [u8]) -> IResult<&'a [u8], O, E>,
{
    delimited(multispace0, inner, multispace0)
}

/// Parse the results of two sub-parsers contained in a structure like ( A , B )
pub(crate) fn pair_values<'a, F: 'a, G: 'a, O1, O2, E: nom::error::ParseError<&'a [u8]>>(
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

pub(crate) fn is_ascii_text_char(chr: u8) -> bool {
    // Space - '~'
    (32u8..=126u8).contains(&chr)
}