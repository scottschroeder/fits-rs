use crate::parser::type_forms::bin_tform;

use super::Header;
use super::Keyword;
use super::Value;
use super::ValueRetrievalError;
use std::str::FromStr;

/// Errors dealing with FITS Extension Tables
#[derive(Debug)]
pub enum TableError<'a> {
    /// Mismatch between expected extension type and header
    IncorrectExtension,
    /// An expected property of the header was not defined
    PropertyNotDefined(Keyword, ValueRetrievalError),
    UnexpectedValue(Keyword, Value<'a>),
    InvalidFormString(Keyword, &'a str),
}

pub enum ParseFormError {
    InvalidBinType,
}

/// Potential issues and/or inconsistencies in a table header
#[derive(Debug)]
pub enum TableLint {}

/// Common extensions found in FITS
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Extension {
    /// Multi-dimensional array similar to the FITS primary header & data unit
    Image,
    /// Contains rows & columns of data expressed in ASCII
    Table,
    /// Flexible and efficient storing of data structures in binary representation
    /// Each entry is allowed to be a single dimensioned array
    BinTable,
}

struct Image {
    bitpix: i64,
    naxis: Vec<u16>,
}
// PCOUNT = 0
// GCOUNT = 1

struct AsciiForm;
#[derive(Debug)]
struct DisplayFormat;
struct AsciiTable<'a> {
    rows: u16,
    cols: u16,
    tbcol: Vec<u16>,
    tform: Vec<AsciiForm>,
    ttype: Option<Vec<&'a str>>,
    tunit: Option<Vec<&'a str>>,
    scaling: Option<Vec<f64>>,
    zero: Option<Vec<f64>>,
    tdisp: Option<Vec<DisplayFormat>>,
}
// BITPIX = 8
// NAXIS = 2
// PCOUNT = 0
// GCOUNT = 1

// rTa
// r: repeat count, non-neg int specifiying the number of elements (default=1)
// T: data type letter code (for (P | Q), r must be 0,1)
// a: optional?
// b: number of bytes for a type T
// total bytes in a row: sum([r * b for r,b in tfields])

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BinForm {
    pub repeat: u16,
    pub bintype: BinType,
}

/// A code indicating the type of a bintable field
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinType {
    /// Logical
    L,
    /// Bit
    X,
    /// Unsigned byte
    B,
    /// 16-bit integer
    I,
    /// 32-bit integer
    J,
    /// 64-bit integer
    K,
    /// Character
    A,
    /// 32-bit float
    E,
    /// 64-bit float
    D,
    /// 32-bit complex
    C,
    /// 64-bit complex
    M,
    /// Array Descriptor (32-bit)
    P,
    /// Array Descriptor (64-bit)
    Q,
}

impl FromStr for BinType {
    type Err = ParseFormError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "L" => BinType::L,
            "X" => BinType::X,
            "B" => BinType::B,
            "I" => BinType::I,
            "J" => BinType::J,
            "K" => BinType::K,
            "A" => BinType::A,
            "E" => BinType::E,
            "D" => BinType::D,
            "C" => BinType::C,
            "M" => BinType::M,
            "P" => BinType::P,
            "Q" => BinType::Q,
            _ => return Err(ParseFormError::InvalidBinType),
        })
    }
}

impl BinType {
    fn size(self) -> u8 {
        match self {
            BinType::L => 1,
            BinType::X => 1, // TODO check if this is right
            BinType::B => 1,
            BinType::I => 2,
            BinType::J => 4,
            BinType::K => 8,
            BinType::A => 1,
            BinType::E => 4,
            BinType::D => 8,
            BinType::C => 8,
            BinType::M => 16,
            BinType::P => 8,
            BinType::Q => 16,
        }
    }
}

#[derive(Debug)]
pub struct BinTable<'a> {
    rows: usize,        // NAXIS1
    cols: usize,        // NAXIS2
    heap_size: usize, // PCOUNT is number of bytes that follow the table
    tform: Vec<BinForm>,

    ttype: Option<Vec<&'a str>>,
    tunit: Option<Vec<&'a str>>,

    // not used with A L or X
    // for P & Q, this is applied to values in the heap
    scaling: Option<Vec<f64>>,

    // Mostly the same as scaling
    // Also used when storing unsigned ints, see table 19
    // this is used to convert between signed/unsigned ints
    zero: Option<Vec<f64>>,

    null: Option<i64>,
    tdisp: Option<Vec<DisplayFormat>>,

    theap: usize, // number of bytes between start of data table, and heap

    tdim: Option<Vec<()>>,
}

fn get_str<'a>(header: &Header<'a>, keyword: Keyword) -> Result<&'a str, TableError<'a>> {
    header
        .str_value_of(&keyword)
        .map_err(|e| TableError::PropertyNotDefined(keyword, e))
}

fn get_int<'a>(header: &Header<'a>, keyword: Keyword) -> Result<i64, TableError<'a>> {
    header
        .integer_value_of(&keyword)
        .map_err(|e| TableError::PropertyNotDefined(keyword, e))
}

fn get_uint<'a>(header: &Header<'a>, keyword: Keyword) -> Result<u64, TableError<'a>> {
    let val = header
        .value_of(&keyword)
        .map_err(|e| TableError::PropertyNotDefined(keyword.clone(), e))?;
    if let Value::Integer(i) = val {
        if i < 0 {
            Err(TableError::UnexpectedValue(keyword, val))
        } else {
            Ok(i as u64)
        }
    } else {
        Err(TableError::PropertyNotDefined(
            keyword,
            ValueRetrievalError::NotAnInteger,
        ))
    }
}

fn get_value<'a>(header: &Header<'a>, keyword: Keyword) -> Result<Value<'a>, TableError<'a>> {
    header
        .value_of(&keyword)
        .map_err(|e| TableError::PropertyNotDefined(keyword, e))
}

impl<'a> BinTable<'a> {
    pub fn new(header: &Header<'a>) -> Result<BinTable<'a>, TableError<'a>> {
        // Verify required values for a BINTABLE
        if get_str(header, Keyword::XTENSION)? != "BINTABLE" {
            return Err(TableError::IncorrectExtension);
        }
        let bitpix = get_value(header, Keyword::BITPIX)?;
        if bitpix != Value::Integer(8) {
            return Err(TableError::UnexpectedValue(Keyword::BITPIX, bitpix));
        }
        let naxis = get_value(header, Keyword::NAXIS)?;
        if naxis != Value::Integer(2) {
            return Err(TableError::UnexpectedValue(Keyword::NAXIS, naxis));
        }
        let gcount = get_value(header, Keyword::GCOUNT)?;
        if gcount != Value::Integer(1) {
            return Err(TableError::UnexpectedValue(Keyword::GCOUNT, gcount));
        }

        let rows = get_uint(header, Keyword::NAXISn(1))? as usize;
        let cols = get_uint(header, Keyword::NAXISn(2))? as usize;
        let heap_size = get_uint(header, Keyword::PCOUNT)? as usize;

        let tfields = get_uint(header, Keyword::TFIELDS)? as u16;

        let mut tform = Vec::with_capacity(tfields as usize);
        let mut ttype = Vec::with_capacity(tfields as usize);

        for field_idx in 1..(tfields + 1) {
            let tformn = Keyword::TFORMn(field_idx);
            let tform_encoded = get_str(header, tformn.clone())?;
            let (_, tform_idx) = bin_tform(tform_encoded)
                .map_err(|_| TableError::InvalidFormString(tformn, tform_encoded))?;
            tform.push(tform_idx);

            if let Ok(ttype_idx) = get_str(header, Keyword::TTYPEn(field_idx)) {
                ttype.push(ttype_idx);
            }
        }
        let ttype = if ttype.len() == tfields as usize {
            Some(ttype)
        } else {
            None
        };

        let theap = if let Ok(t) = get_uint(header, Keyword::THEAP) {
            t as usize
        } else if heap_size == 0 {
            0
        } else {
            rows as usize * cols as usize
        };

        Ok(BinTable {
            rows,
            cols,
            heap_size,
            tform,
            ttype,
            tunit: None,
            scaling: None,
            zero: None,
            null: None,
            tdisp: None,
            theap,
            tdim: None,
        })
    }
}
