//! Types to help with parsing fits files
use crate::{
    fits::FITS_BLOCK_SIZE,
    parser::header,
    types::{Header, HeaderRecord},
};

/// Parse a fits header
#[derive(Debug)]
pub struct HeaderParser<'a> {
    records: Vec<HeaderRecord<'a>>,
    parse_more: bool,
    consumed: usize,
}

/// a
pub enum ParseOutcome<'a> {
    /// a
    Ok(&'a [u8]),
    /// a
    Complete(&'a [u8]),
    /// a
    Error(nom::Err<nom::error::Error<&'a [u8]>>),
}

impl<'a> HeaderParser<'a> {
    /// Create parser
    pub fn new() -> HeaderParser<'a> {
        HeaderParser {
            records: Vec::new(),
            parse_more: true,
            consumed: 0,
        }
    }

    fn is_complete(&self) -> bool {
        !self.parse_more && self.consumed % FITS_BLOCK_SIZE == 0
    }

    /// Parse a single header block from input
    pub fn parse_some(
        &mut self,
        mut input: &'a [u8],
    ) -> Result<&'a [u8], nom::Err<nom::error::Error<&'a [u8]>>> {
        loop {
            match self.parse_record(input) {
                ParseOutcome::Ok(r) => input = r,
                ParseOutcome::Complete(c) => return Ok(c),
                ParseOutcome::Error(e) => return Err(e),
            }
        }
    }

    /// Convert this into the `Header` type
    pub fn into_header(self) -> Header<'a> {
        Header::new(self.records)
    }

    /// parse single record from buf
    pub fn parse_record(&mut self, input: &'a [u8]) -> ParseOutcome<'a> {
        match header::header_record(input) {
            Ok((remainder, record)) => {
                match (self.parse_more, &record) {
                    (true, HeaderRecord::EndRecord) => self.parse_more = false,
                    (false, HeaderRecord::BlankRecord) => {}
                    (false, _) => panic!("tried to parse more records after header ended"),
                    _ => {}
                }
                self.consumed += input.len() - remainder.len();
                self.records.push(record);
                if self.is_complete() {
                    ParseOutcome::Complete(remainder)
                } else {
                    ParseOutcome::Ok(remainder)
                }
            }
            Err(e) => ParseOutcome::Error(e),
        }
    }
}
