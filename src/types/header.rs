use crate::{fits::FITS_BLOCK_SIZE, types::Keyword};
use std::fmt::{Display, Error, Formatter};

/// A FITS header
#[derive(Debug, PartialEq)]
pub struct Header<'a> {
    records: Vec<HeaderRecord<'a>>,
    start: usize,
    length: usize,
}

impl<'a> Header<'a> {
    /// Create a Header with a given set of keyword_records
    pub fn new(records: Vec<HeaderRecord<'a>>, start: usize, length: usize) -> Header<'a> {
        Header {
            records,
            start,
            length,
        }
    }

    fn header_end_position(&self) -> usize {
        self.start + self.length
    }

    /// Position where the next header in the file may start
    ///
    /// There may or may not actually be a header at this location
    pub(crate) fn next_header(&self) -> usize {
        self.header_end_position() + self.data_array_bits() / 8
    }

    /// The (start, end) positions of the data array described by this header
    pub(crate) fn data_array_boundaries(&self) -> (usize, usize) {
        (self.header_end_position(), self.next_header())
    }

    /// Determines the size in *bits* of the data array following this header.
    pub fn data_array_bits(&self) -> usize {
        if self.is_primary() {
            lmle(self.primary_data_array_size(), FITS_BLOCK_SIZE * 8)
        } else {
            lmle(self.extention_data_array_size(), FITS_BLOCK_SIZE * 8)
        }
    }

    fn keyword_records(&self) -> impl Iterator<Item = &KeywordRecord<'a>> {
        self.records.iter().filter_map(|r| match r {
            HeaderRecord::KeywordRecord(k) => Some(k),
            _ => None,
        })
    }

    fn is_primary(&self) -> bool {
        self.has_keyword_record(&Keyword::SIMPLE)
    }

    fn has_keyword_record(&self, keyword: &Keyword) -> bool {
        for keyword_record in self.keyword_records() {
            if *keyword == keyword_record.keyword {
                return true;
            }
        }
        false
    }

    fn primary_data_array_size(&self) -> usize {
        (self
            .integer_value_of(&Keyword::BITPIX)
            .unwrap_or(0i64)
            .abs()
            * self.naxis_product()) as usize
    }

    fn extention_data_array_size(&self) -> usize {
        (self
            .integer_value_of(&Keyword::BITPIX)
            .unwrap_or(0i64)
            .abs()
            * self.integer_value_of(&Keyword::GCOUNT).unwrap_or(1i64)
            * (self.integer_value_of(&Keyword::PCOUNT).unwrap_or(0i64) + self.naxis_product()))
            as usize
    }

    /// Get the value of a keyword as an `i64`
    pub fn integer_value_of(&self, keyword: &Keyword) -> Result<i64, ValueRetrievalError> {
        self.value_of(keyword).and_then(|value| match value {
            Value::Integer(n) => Ok(n),
            _ => Err(ValueRetrievalError::NotAnInteger),
        })
    }

    /// Get the value of a keyword as a `str`
    pub fn str_value_of(&self, keyword: &Keyword) -> Result<&'a str, ValueRetrievalError> {
        self.value_of(keyword).and_then(|value| match value {
            Value::CharacterString(s) => Ok(s),
            _ => Err(ValueRetrievalError::NotAString),
        })
    }

    /// Get the value of a keyword
    pub fn value_of(&self, keyword: &Keyword) -> Result<Value<'a>, ValueRetrievalError> {
        if self.has_keyword_record(keyword) {
            for keyword_record in self.keyword_records() {
                if keyword_record.keyword == *keyword {
                    return Ok(keyword_record.value.clone());
                }
            }
        }
        Err(ValueRetrievalError::KeywordNotPresent)
    }

    fn naxis_product(&self) -> i64 {
        let limit = self.integer_value_of(&Keyword::NAXIS).unwrap_or(0i64);
        if limit > 0 {
            let mut product = 1i64;
            for n in 0..limit {
                let naxisn = Keyword::NAXISn((n + 1i64) as u16);
                product *= self
                    .integer_value_of(&naxisn)
                    .unwrap_or_else(|_| panic!("NAXIS{} should be defined", n));
            }
            product
        } else {
            0i64
        }
    }
}

/// When asking for a value, these things can go wrong.
#[derive(Debug)]
pub enum ValueRetrievalError {
    /// The value associated with this keyword is not an integer.
    NotAnInteger,
    /// The value associated with this keyword is not a string.
    NotAString,
    /// The value associated with this keyword is not a bool.
    NotABool,
    /// There is no value associated with this keyword.
    ValueUndefined,
    /// The keyword is not present in the header.
    KeywordNotPresent,
}

/// A value record contains information about a FITS header.
/// It maps to one of several types of header records
#[derive(Debug, PartialEq)]
pub enum HeaderRecord<'a> {
    /// A `KeywordRecord` that maps a keyword to a value
    KeywordRecord(KeywordRecord<'a>),
    /// A `CommentaryRecord` that contains text data
    CommentaryRecord(CommentaryRecord<'a>),
    /// A terminal record, indicating the end of a section
    EndRecord,
    /// A placeholder for blank records
    BlankRecord(Option<&'a str>),
}

impl<'a> Display for Header<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        for hr in &self.records {
            if let HeaderRecord::BlankRecord(None) = hr {
                continue;
            }
            writeln!(f, "{}", hr)?;
        }
        Ok(())
    }
}

impl<'a> Display for HeaderRecord<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            HeaderRecord::KeywordRecord(v) => write!(f, "{}", v),
            HeaderRecord::CommentaryRecord(c) => write!(f, "{}", c),
            HeaderRecord::EndRecord => write!(f, "{}", Keyword::END),
            HeaderRecord::BlankRecord(None) => write!(f, ""),
            HeaderRecord::BlankRecord(Some(s)) => write!(f, "/ {}", s),
        }
    }
}

/// A value record contains information about a FITS header. It consists of a
/// keyword, the corresponding value and an optional comment.
#[derive(Debug, PartialEq)]
pub struct KeywordRecord<'a> {
    /// The keyword of this record.
    keyword: Keyword,
    /// The value of this record.
    value: Value<'a>,
    /// The comment of this record.
    comment: Option<&'a str>,
}

impl<'a> KeywordRecord<'a> {
    /// Create a `KeywordRecord` from a specific `Keyword`.
    pub fn new(keyword: Keyword, value: Value<'a>, comment: Option<&'a str>) -> KeywordRecord<'a> {
        KeywordRecord {
            keyword,
            value,
            comment,
        }
    }
}

impl<'a> Display for KeywordRecord<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{}= {:?}/{}",
            self.keyword,
            self.value,
            self.comment.unwrap_or("")
        )
    }
}

/// A commentary record contains information about a FITS header. It consists of a
/// keyword, the corresponding commentary and an optional comment.
#[derive(Debug, PartialEq)]
pub struct CommentaryRecord<'a> {
    /// The keyword of this record.
    keyword: Keyword,
    /// The comment of this record.
    commentary: Option<&'a str>,
}

impl<'a> CommentaryRecord<'a> {
    /// Create a `KeywordRecord` from a specific `Keyword`.
    pub fn new(keyword: Keyword, commentary: Option<&'a str>) -> CommentaryRecord<'a> {
        CommentaryRecord {
            keyword,
            commentary,
        }
    }
}

impl<'a> Display for CommentaryRecord<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "{} {}", self.keyword, self.commentary.unwrap_or(""))
    }
}

/// The possible values of a KeywordRecord.
#[derive(Debug, PartialEq, Clone)]
pub enum Value<'a> {
    /// A string enclosed in single quotes `'`.
    CharacterString(&'a str),
    /// A logical constant signified by either an uppercase `F` or an uppercase `T`.
    Logical(bool),
    /// An optionally signed decimal integer.
    Integer(i64),
    /// Complex integer represented by a real and imaginary component.
    ComplexInteger((i64, i64)),
    /// Fixed format real floating point number.
    Real(f64),
    /// Complex number represented by a real and imaginary component.
    Complex((f64, f64)),
    /// When a value is not present
    Undefined,
}

/// For input n and k, finds the least multiple of k such that n <= q*k and
/// (q-1)*k < n
fn lmle(n: usize, k: usize) -> usize {
    let (q, r) = (n / k, n % k);
    if r == 0 {
        q * k
    } else {
        (q + 1) * k
    }
}

#[cfg(test)]
mod tests {
    use crate::fits::KEYWORD_LINE_LENGTH;

    use super::*;

    fn build_test_header(records: Vec<HeaderRecord>) -> Header {
        let expected_len = records.len() * KEYWORD_LINE_LENGTH;
        Header::new(records, 0, expected_len)
    }

    #[test]
    fn header_constructed_from_the_new_function_should_eq_hand_construction() {
        assert_eq!(
            Header {
                records: vec!(
                    HeaderRecord::KeywordRecord(KeywordRecord::new(
                        Keyword::SIMPLE,
                        Value::Logical(true),
                        Option::None
                    )),
                    HeaderRecord::KeywordRecord(KeywordRecord::new(
                        Keyword::NEXTEND,
                        Value::Integer(0i64),
                        Option::Some("no extensions")
                    )),
                ),
                start: 0,
                length: KEYWORD_LINE_LENGTH * 2,
            },
            Header::new(
                vec!(
                    HeaderRecord::KeywordRecord(KeywordRecord::new(
                        Keyword::SIMPLE,
                        Value::Logical(true),
                        Option::None
                    )),
                    HeaderRecord::KeywordRecord(KeywordRecord::new(
                        Keyword::NEXTEND,
                        Value::Integer(0i64),
                        Option::Some("no extensions")
                    )),
                ),
                0,
                KEYWORD_LINE_LENGTH * 2
            )
        );
    }

    #[test]
    fn keyword_record_constructed_from_the_new_function_should_eq_hand_construction() {
        assert_eq!(
            KeywordRecord {
                keyword: Keyword::ORIGIN,
                value: Value::Undefined,
                comment: Option::None
            },
            KeywordRecord::new(Keyword::ORIGIN, Value::Undefined, Option::None)
        );
    }

    #[test]
    fn primary_header_should_determine_correct_data_array_size() {
        let header = build_test_header(vec![
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::SIMPLE,
                Value::Logical(true),
                Option::None,
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::BITPIX,
                Value::Integer(8i64),
                Option::None,
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::NAXIS,
                Value::Integer(2i64),
                Option::None,
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::NAXISn(1u16),
                Value::Integer(3i64),
                Option::None,
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::NAXISn(2u16),
                Value::Integer(5i64),
                Option::None,
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::END,
                Value::Undefined,
                Option::None,
            )),
        ]);

        assert_eq!(header.data_array_bits(), (FITS_BLOCK_SIZE * 8) as usize);
    }

    #[test]
    fn extension_header_should_determine_correct_data_array_size() {
        let header = build_test_header(vec![
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::XTENSION,
                Value::CharacterString("BINTABLE"),
                Option::None,
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::BITPIX,
                Value::Integer(128i64),
                Option::None,
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::NAXIS,
                Value::Integer(2i64),
                Option::None,
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::NAXISn(1u16),
                Value::Integer(3i64),
                Option::None,
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::NAXISn(2u16),
                Value::Integer(5i64),
                Option::None,
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::GCOUNT,
                Value::Integer(7i64),
                Option::None,
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::PCOUNT,
                Value::Integer(11i64),
                Option::None,
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::END,
                Value::Undefined,
                Option::None,
            )),
        ]);

        assert_eq!(header.data_array_bits(), 2 * (FITS_BLOCK_SIZE * 8) as usize);
    }
}
