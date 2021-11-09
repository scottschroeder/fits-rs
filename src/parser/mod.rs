//! The parser module is responsible for parsing FITS files.
//! This parser was created following the FITS 3.0 standard.
//! Specifically https://www.aanda.org/articles/aa/pdf/2010/16/aa15362-10.pdf
//! using Appendix A.
//!
//! We deviate from their organizational structure to make header END and <blank>
//! records easier to reason about.
mod header;
pub mod stream_parser;
mod util;
use crate::types::HeaderRecord;
use nom::IResult;

/// Parse the entire header data out of a FITS bytestream
pub fn parse_header(input: &[u8]) -> IResult<&[u8], Vec<HeaderRecord>> {
    // many0(keyword_record)(input)
    header::header(input)
}
