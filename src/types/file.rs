use super::Header;

/// Representation of a FITS file.
#[derive(Debug, PartialEq)]
pub struct Fits<'a> {
    /// all the headers of a FITS file
    pub headers: Vec<Header<'a>>,
}
