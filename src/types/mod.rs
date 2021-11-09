//! The types modules describes all the structures to express FITS files.

mod file;
mod header;
mod keyword;

pub use file::Fits;
pub use header::{
    CommentaryRecord, Header, HeaderRecord, KeywordRecord, Value, ValueRetrievalError,
};
pub use keyword::Keyword;
