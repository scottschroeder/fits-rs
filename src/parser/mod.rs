//! The parser module is responsible for parsing FITS files.
//! This parser was created following the FITS 3.0 standard.
//! Specifically https://www.aanda.org/articles/aa/pdf/2010/16/aa15362-10.pdf
//! using Appendix A.
//!
//! We deviate from their organizational structure to make header END and <blank>
//! records easier to reason about.
mod header;
mod util;

mod helper;
use self::helper::HeaderParser;
use crate::types::Fits;

type ParseError<'a> = nom::Err<nom::error::Error<&'a [u8]>>;

/// Will parse data from a FITS file into a `Fits` structure
pub fn parse(input: &[u8]) -> Result<Fits, ParseError> {
    let mut headers = Vec::new();
    let mut start = 0;
    loop {
        let segment = &input[start..];
        if segment.is_empty() {
            // We ran out of file, so we are done
            break;
        }
        let mut helper = HeaderParser::new(start);
        helper.parse_header(segment)?;
        let header = helper.into_header();
        start = header.next_header();
        headers.push(header);
    }
    Ok(Fits { headers })
}
