use super::Header;
use std::fmt;

/// Representation of a FITS file.
#[derive(Debug, PartialEq)]
pub struct Fits<'a> {
    /// all the headers of a FITS file
    pub hdu: Vec<HDU<'a>>,
}

/// Representation a header and data section
#[derive(PartialEq)]
pub struct HDU<'a> {
    pub header: Header<'a>,
    pub data: &'a [u8],
}

impl<'a> fmt::Debug for HDU<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HDU")
            .field("header", &self.header)
            .field("data (len)", &self.data.len())
            .finish()
    }
}

impl<'a> HDU<'a> {
    pub(crate) fn new(header: Header<'a>, input: &'a [u8]) -> HDU<'a> {
        let (start, end) = header.data_array_boundaries();
        HDU {
            header,
            data: &input[start..end],
        }
    }
}
