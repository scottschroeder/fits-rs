//! The parser module is responsible for parsing FITS files.





/// Problems that could occur when parsing a `str` for a Value::Logical are enumerated here.
pub enum ParseLogicalConstantError {
    /// When encountering anything other than `"T"` or `"F"`.
    UnknownConstant
}

/// Reasons for converting to a f64 from a parse triple (left, _, right) to fail.
pub enum RealParseError {
    /// When left is not parse-able as `str`.
    IntegerPartUnparseable,
    /// When right is not parse-able as `str`.
    FractionalPartUnparseable,
    /// When the combination is not a `f64`.
    NotARealNumber,
}


