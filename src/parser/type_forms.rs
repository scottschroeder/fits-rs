use crate::parser::util::{exact_length, is_ascii_text_char, pair_values, ws};
use crate::types::BinForm;
use crate::types::BinType;
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
/*
"16A"
"0A"
"4A"
"2A"
"8A"
"8A"
"0A"
"4A"
"4A"
"8A"
"12A"
"1B"
"1E"
*/

pub(crate) fn bin_tform(input: &str) -> IResult<&str, BinForm> {
    map_res(
        tuple((
            opt(repeat_count),
            take(1usize),
            opt(take_while(is_allowed_ascii_char)),
        )),
        |(a, b, _)| {
            let repeat = a.unwrap_or(1);
            BinType::from_str(b).map(|bintype| BinForm { repeat, bintype })
        },
    )(input)
}

fn repeat_count(input: &str) -> IResult<&str, u16> {
    map_res(take_while1(is_digit_char), u16::from_str)(input)
}

fn is_digit_char(c: char) -> bool {
    is_digit(c as u8)
}

fn is_allowed_ascii_char(c: char) -> bool {
    is_ascii_text_char(c as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_valid_binary_tform() {
        let valid_pairs = &[
            (
                "0A",
                BinForm {
                    repeat: 0,
                    bintype: BinType::A,
                },
            ),
            (
                "12A",
                BinForm {
                    repeat: 12,
                    bintype: BinType::A,
                },
            ),
            (
                "16A",
                BinForm {
                    repeat: 16,
                    bintype: BinType::A,
                },
            ),
            (
                "1B",
                BinForm {
                    repeat: 1,
                    bintype: BinType::B,
                },
            ),
            (
                "1E",
                BinForm {
                    repeat: 1,
                    bintype: BinType::E,
                },
            ),
            (
                "2A",
                BinForm {
                    repeat: 2,
                    bintype: BinType::A,
                },
            ),
            (
                "4A",
                BinForm {
                    repeat: 4,
                    bintype: BinType::A,
                },
            ),
            (
                "8A",
                BinForm {
                    repeat: 8,
                    bintype: BinType::A,
                },
            ),
        ];
        for (input, expected) in valid_pairs {
            let (_, k) = bin_tform(input).unwrap();
            assert_eq!(k, *expected);
        }
    }
}
