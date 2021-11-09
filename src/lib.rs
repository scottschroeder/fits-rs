#![warn(missing_docs)] // TODO deny
//! An encoder and decoder for FITS images.
//!
//! The *Flexible Image Transport System* ([FITS](https://en.wikipedia.org/wiki/FITS)) is
//! > an open standard defining a digital file format useful for storage,
//! > transmission and processing of scientific and other images.

pub mod parser;
pub mod types;
mod fits {
    /// All Keyword/Value/Comment lines are this fixed length
    pub(crate) const KEYWORD_LINE_LENGTH: usize = 80;

    /// All segments are in mulitples of this many bytes
    pub(crate) const FITS_BLOCK_SIZE: usize = 36 * KEYWORD_LINE_LENGTH; // 2880
}
