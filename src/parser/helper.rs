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
    start: usize,
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
    pub fn new(start: usize) -> HeaderParser<'a> {
        HeaderParser {
            records: Vec::new(),
            parse_more: true,
            start,
            consumed: 0,
        }
    }

    fn is_complete(&self) -> bool {
        !self.parse_more && self.consumed % FITS_BLOCK_SIZE == 0
    }

    /// Parse a single header from input
    /// Will return an error if we did not sucessfully parse an
    /// entire well-formed header.
    ///
    /// If we return an error, you can still inspect this object
    /// to see partial results.
    pub fn parse_header(
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
        Header::new(self.records, self.start, self.consumed)
    }

    /// parse single record from buf
    pub fn parse_record(&mut self, input: &'a [u8]) -> ParseOutcome<'a> {
        match header::header_record(input) {
            Ok((remainder, record)) => {
                match (self.parse_more, &record) {
                    (true, HeaderRecord::EndRecord) => self.parse_more = false,
                    (false, HeaderRecord::BlankRecord(_)) => {}
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

#[cfg(test)]
mod tests {
    use crate::types::{Keyword, KeywordRecord, Value};

    use super::*;

    #[test]
    fn header_should_parse_a_primary_header() {
        let data =
            include_bytes!("../../assets/images/k2-trappist1-unofficial-tpf-long-cadence.fits");
        let mut helper = HeaderParser::new(0);
        helper.parse_header(data).unwrap();
        let expect = long_cadence_header();
        for (idx, (r, e)) in helper.records.into_iter().zip(expect).enumerate() {
            assert_eq!(r, e, "{}", idx)
        }
    }

    fn long_cadence_header<'a>() -> Vec<HeaderRecord<'a>> {
        vec![
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::SIMPLE,
                Value::Logical(true),
                Option::Some("conforms to FITS standards"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::BITPIX,
                Value::Integer(8i64),
                Option::Some("array data type"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::NAXIS,
                Value::Integer(0i64),
                Option::Some("number of array dimensions"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::EXTEND,
                Value::Logical(true),
                Option::Some("file contains extensions"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::NEXTEND,
                Value::Integer(2i64),
                Option::Some("number of standard extensions"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::EXTNAME,
                Value::CharacterString("PRIMARY"),
                Option::Some("name of extension"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::EXTVER,
                Value::Integer(1i64),
                Option::Some("extension version number (not format version)"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::ORIGIN,
                Value::CharacterString("Unofficial data product"),
                Option::Some("institution responsible for creating this"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::DATE,
                Value::CharacterString("2017-03-08"),
                Option::Some("file creation date."),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::CREATOR,
                Value::CharacterString("kadenza"),
                Option::Some("pipeline job and program u"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::PROCVER,
                Value::CharacterString("2.1.dev"),
                Option::Some("SW version"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::FILEVER,
                Value::CharacterString("0.0"),
                Option::Some("file format version"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::TIMVERSN,
                Value::CharacterString(""),
                Option::Some("OGIP memo number for file format"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::TELESCOP,
                Value::CharacterString("Kepler"),
                Option::Some("telescope"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::INSTRUME,
                Value::CharacterString("Kepler Photometer"),
                Option::Some("detector type"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::OBJECT,
                Value::CharacterString("EPIC 200164267"),
                Option::Some("string version of target id"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::KEPLERID,
                Value::Integer(200164267i64),
                Option::Some("unique Kepler target identifier"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::CHANNEL,
                Value::Integer(68i64),
                Option::Some("CCD channel"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::MODULE,
                Value::Integer(19i64),
                Option::Some("CCD module"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::OUTPUT,
                Value::Integer(4i64),
                Option::Some("CCD output"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::CAMPAIGN,
                Value::CharacterString(""),
                Option::Some("Observing campaign number"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::DATA_REL,
                Value::CharacterString(""),
                Option::Some("data release version number"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::OBSMODE,
                Value::CharacterString("long cadence"),
                Option::Some("observing mode"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::MISSION,
                Value::CharacterString("K2"),
                Option::Some("Mission name"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::TTABLEID,
                Value::CharacterString(""),
                Option::Some("target table id"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::RADESYS,
                Value::CharacterString("ICRS"),
                Option::Some("reference frame of celestial coordinates"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::RA_OBJ,
                Value::CharacterString(""),
                Option::Some("[deg] right ascension"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::DEC_OBJ,
                Value::CharacterString(""),
                Option::Some("[deg] declination"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::EQUINOX,
                Value::Real(2000.0f64),
                Option::Some("equinox of celestial coordinate system"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::PMRA,
                Value::Undefined,
                Option::Some("[arcsec/yr] RA proper motion"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::PMDEC,
                Value::Undefined,
                Option::Some("[arcsec/yr] Dec proper motion"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::PMTOTAL,
                Value::Undefined,
                Option::Some("[arcsec/yr] total proper motion"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::PARALLAX,
                Value::Undefined,
                Option::Some("[arcsec] parallax"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::GLON,
                Value::Undefined,
                Option::Some("[deg] galactic longitude"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::GLAT,
                Value::Undefined,
                Option::Some("[deg] galactic latitude"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::GMAG,
                Value::Undefined,
                Option::Some("[mag] SDSS g band magnitude"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::RMAG,
                Value::Undefined,
                Option::Some("[mag] SDSS r band magnitude"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::IMAG,
                Value::Undefined,
                Option::Some("[mag] SDSS i band magnitude"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::ZMAG,
                Value::Undefined,
                Option::Some("[mag] SDSS z band magnitude"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::JMAG,
                Value::Undefined,
                Option::Some("[mag] J band magnitude from 2MASS"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::HMAG,
                Value::Undefined,
                Option::Some("[mag] H band magnitude from 2MASS"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::KMAG,
                Value::Undefined,
                Option::Some("[mag] K band magnitude from 2MASS"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::KEPMAG,
                Value::Undefined,
                Option::Some("[mag] Kepler magnitude (Kp)"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::GRCOLOR,
                Value::Undefined,
                Option::Some("[mag] (g-r) color, SDSS bands"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::JKCOLOR,
                Value::Undefined,
                Option::Some("[mag] (J-K) color, 2MASS bands"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::GKCOLOR,
                Value::Undefined,
                Option::Some("[mag] (g-K) color, SDSS g - 2MASS K"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::TEFF,
                Value::Undefined,
                Option::Some("[K] Effective temperature"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::LOGG,
                Value::Undefined,
                Option::Some("[cm/s2] log10 surface gravity"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::FEH,
                Value::Undefined,
                Option::Some("[log10([Fe/H])]  metallicity"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::EBMINUSV,
                Value::Undefined,
                Option::Some("[mag] E(B-V) reddening"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::AV,
                Value::Undefined,
                Option::Some("[mag] A_v extinction"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::RADIUS,
                Value::Undefined,
                Option::Some("[solar radii] stellar radius"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::TMINDEX,
                Value::Undefined,
                Option::Some("unique 2MASS catalog ID"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::CHECKSUM,
                Value::CharacterString("7k7A7h637h697h69"),
                Option::Some("HDU checksum updated 2017-03-08T02:47:56"),
            )),
            HeaderRecord::KeywordRecord(KeywordRecord::new(
                Keyword::DATASUM,
                Value::CharacterString("0"),
                Option::Some("data unit checksum updated 2017-03-08T02:47:56"),
            )),
            HeaderRecord::EndRecord,
            HeaderRecord::BlankRecord(None),
            HeaderRecord::BlankRecord(None),
            HeaderRecord::BlankRecord(None),
            HeaderRecord::BlankRecord(None),
            HeaderRecord::BlankRecord(None),
            HeaderRecord::BlankRecord(None),
            HeaderRecord::BlankRecord(None),
            HeaderRecord::BlankRecord(None),
            HeaderRecord::BlankRecord(None),
            HeaderRecord::BlankRecord(None),
            HeaderRecord::BlankRecord(None),
            HeaderRecord::BlankRecord(None),
            HeaderRecord::BlankRecord(None),
            HeaderRecord::BlankRecord(None),
            HeaderRecord::BlankRecord(None),
        ]
    }
}
