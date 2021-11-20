//! The types modules describes all the structures to express FITS files.

mod file;
mod header;
mod keyword;
mod extension;

pub use file::Fits;
pub use file::HDU;
pub use header::{
    CommentaryRecord, Header, HeaderRecord, KeywordRecord, Value, ValueRetrievalError,
};
pub use keyword::Keyword;
pub use extension::BinType;
pub use extension::BinForm;
pub use extension::BinTable;
pub use extension::TableError;
