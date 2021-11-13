use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use self::keyword_text::KeywordText;

mod keyword_text {
    use std::fmt;
    use std::ops::Deref;
    use std::str;

    /// A stack-allocated string to store unrecognized keywords.
    ///
    /// Limited to exactly 8 bytes.
    #[derive(Clone, Copy, PartialEq)]
    pub struct KeywordText {
        buf: [u8; 8],
        len: usize,
    }

    impl KeywordText {
        pub(crate) fn new(keyword: &str) -> KeywordText {
            let len = keyword.as_bytes().len();
            assert!(
                len <= 8,
                "keyword can not store string larger than 8 bytes: {:?} has {} bytes",
                keyword,
                len
            );
            let mut buf = [0; 8];
            buf[0..len].copy_from_slice(keyword.as_bytes());
            KeywordText { buf, len }
        }

        pub fn as_str(&self) -> &str {
            // This struct is in its own module, and the only way to instantiate it
            // is to use the `new` function, which only accepts a `&str`.
            unsafe { str::from_utf8_unchecked(&self.buf[..self.len]) }
        }
    }

    impl Deref for KeywordText {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            self.as_str()
        }
    }

    impl fmt::Debug for KeywordText {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.as_str().fmt(f)
        }
    }

    impl<T: AsRef<str>> From<T> for KeywordText {
        fn from(s: T) -> Self {
            let s = s.as_ref();
            KeywordText::new(s)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn create_keyword_full() {
            let s = "deadbeef";
            assert_eq!(KeywordText::new(s).as_str(), s);
        }
        #[test]
        fn create_keyword_partial() {
            let s = "fitskey";
            assert_eq!(KeywordText::new(s).as_str(), s);
        }
        #[test]
        fn create_keyword_empty() {
            let s = "";
            assert_eq!(KeywordText::new(s).as_str(), s);
        }
        #[test]
        fn keyword_deref_str_check() {
            let s = "fitskey";
            let k = KeywordText::new(s);
            assert_eq!(k.find("t"), Some(2));
        }
    }
}

/// The various keywords that can be found in headers.
#[derive(Debug, Clone, PartialEq)]
#[allow(non_camel_case_types, missing_docs)]
pub enum Keyword {
    AV,
    BITPIX,
    CAMPAIGN,
    CHANNEL,
    CHECKSUM,
    COMMENT,
    CREATOR,
    DATASUM,
    DATA_REL,
    DATE,
    DEC_OBJ,
    EBMINUSV,
    END,
    EQUINOX,
    EXTEND,
    EXTNAME,
    EXTVER,
    FEH,
    FILEVER,
    GCOUNT,
    GKCOLOR,
    GLAT,
    GLON,
    GMAG,
    GRCOLOR,
    HISTORY,
    HMAG,
    IMAG,
    INSTRUME,
    JKCOLOR,
    JMAG,
    KEPLERID,
    KEPMAG,
    KMAG,
    LOGG,
    MISSION,
    MODULE,
    NAXIS,
    NAXISn(u16),
    NEXTEND,
    OBJECT,
    OBSMODE,
    ORIGIN,
    OUTPUT,
    PARALLAX,
    PCOUNT,
    PMDEC,
    PMRA,
    PMTOTAL,
    PROCVER,
    RADESYS,
    RADIUS,
    RA_OBJ,
    RMAG,
    SIMPLE,
    TDIMn(u16),
    TDISPn(u16),
    TEFF,
    TELESCOP,
    TFIELDS,
    TFORMn(u16),
    TIMVERSN,
    THEAP,
    TMINDEX,
    TNULLn(u16),
    TSCALn(u16),
    TTABLEID,
    TTYPEn(u16),
    TUNITn(u16),
    TZEROn(u16),
    XTENSION,
    ZMAG,
    Unrecognized(KeywordText),
}

impl Display for Keyword {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Unrecognized(k) => write!(f, "{}", k.as_str()),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Problems that could occur when parsing a `str` for a Keyword are enumerated here.
#[derive(Debug)]
pub enum ParseKeywordError {
    /// When a str can not be recognized as a keyword, this error will be returned.
    UnknownKeyword,
    /// When `NAXIS<number>` et. al. are parsed where `<number>` is not an actual number.
    NotANumber,
}

impl FromStr for Keyword {
    type Err = ParseKeywordError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim_end() {
            "AV" => Ok(Keyword::AV),
            "BITPIX" => Ok(Keyword::BITPIX),
            "CAMPAIGN" => Ok(Keyword::CAMPAIGN),
            "CHANNEL" => Ok(Keyword::CHANNEL),
            "CHECKSUM" => Ok(Keyword::CHECKSUM),
            "COMMENT" => Ok(Keyword::COMMENT),
            "CREATOR" => Ok(Keyword::CREATOR),
            "DATASUM" => Ok(Keyword::DATASUM),
            "DATA_REL" => Ok(Keyword::DATA_REL),
            "DATE" => Ok(Keyword::DATE),
            "DEC_OBJ" => Ok(Keyword::DEC_OBJ),
            "EBMINUSV" => Ok(Keyword::EBMINUSV),
            "END" => Ok(Keyword::END),
            "EQUINOX" => Ok(Keyword::EQUINOX),
            "EXTEND" => Ok(Keyword::EXTEND),
            "EXTNAME" => Ok(Keyword::EXTNAME),
            "EXTVER" => Ok(Keyword::EXTVER),
            "FEH" => Ok(Keyword::FEH),
            "FILEVER" => Ok(Keyword::FILEVER),
            "GCOUNT" => Ok(Keyword::GCOUNT),
            "GKCOLOR" => Ok(Keyword::GKCOLOR),
            "GLAT" => Ok(Keyword::GLAT),
            "GLON" => Ok(Keyword::GLON),
            "GMAG" => Ok(Keyword::GMAG),
            "GRCOLOR" => Ok(Keyword::GRCOLOR),
            "HISTORY" => Ok(Keyword::HISTORY),
            "HMAG" => Ok(Keyword::HMAG),
            "IMAG" => Ok(Keyword::IMAG),
            "INSTRUME" => Ok(Keyword::INSTRUME),
            "JKCOLOR" => Ok(Keyword::JKCOLOR),
            "JMAG" => Ok(Keyword::JMAG),
            "KEPLERID" => Ok(Keyword::KEPLERID),
            "KEPMAG" => Ok(Keyword::KEPMAG),
            "KMAG" => Ok(Keyword::KMAG),
            "LOGG" => Ok(Keyword::LOGG),
            "MISSION" => Ok(Keyword::MISSION),
            "MODULE" => Ok(Keyword::MODULE),
            "NAXIS" => Ok(Keyword::NAXIS),
            "NEXTEND" => Ok(Keyword::NEXTEND),
            "OBJECT" => Ok(Keyword::OBJECT),
            "OBSMODE" => Ok(Keyword::OBSMODE),
            "ORIGIN" => Ok(Keyword::ORIGIN),
            "OUTPUT" => Ok(Keyword::OUTPUT),
            "PARALLAX" => Ok(Keyword::PARALLAX),
            "PCOUNT" => Ok(Keyword::PCOUNT),
            "PMDEC" => Ok(Keyword::PMDEC),
            "PMRA" => Ok(Keyword::PMRA),
            "PMTOTAL" => Ok(Keyword::PMTOTAL),
            "PROCVER" => Ok(Keyword::PROCVER),
            "RADESYS" => Ok(Keyword::RADESYS),
            "RADIUS" => Ok(Keyword::RADIUS),
            "RA_OBJ" => Ok(Keyword::RA_OBJ),
            "RMAG" => Ok(Keyword::RMAG),
            "SIMPLE" => Ok(Keyword::SIMPLE),
            "TEFF" => Ok(Keyword::TEFF),
            "TELESCOP" => Ok(Keyword::TELESCOP),
            "TFIELDS" => Ok(Keyword::TFIELDS),
            "THEAP" => Ok(Keyword::THEAP),
            "TIMVERSN" => Ok(Keyword::TIMVERSN),
            "TMINDEX" => Ok(Keyword::TMINDEX),
            "TTABLEID" => Ok(Keyword::TTABLEID),
            "XTENSION" => Ok(Keyword::XTENSION),
            "ZMAG" => Ok(Keyword::ZMAG),
            input => {
                let t_dim_constructor = Keyword::TDIMn;
                let t_disp_constructor = Keyword::TDISPn;
                let t_form_constructor = Keyword::TFORMn;
                let naxis_constructor = Keyword::NAXISn;
                let t_null_constructor = Keyword::TNULLn;
                let t_scal_constructor = Keyword::TSCALn;
                let t_type_constructor = Keyword::TTYPEn;
                let t_unit_constructor = Keyword::TUNITn;
                let t_zero_constructor = Keyword::TZEROn;
                let tuples: Vec<(&str, &(dyn Fn(u16) -> Keyword))> = vec![
                    ("TDIM", &t_dim_constructor),
                    ("TDISP", &t_disp_constructor),
                    ("TFORM", &t_form_constructor),
                    ("NAXIS", &naxis_constructor),
                    ("TNULL", &t_null_constructor),
                    ("TSCAL", &t_scal_constructor),
                    ("TTYPE", &t_type_constructor),
                    ("TUNIT", &t_unit_constructor),
                    ("TZERO", &t_zero_constructor),
                ];
                let special_cases: Vec<PrefixedKeyword> = tuples
                    .into_iter()
                    .map(|(prefix, constructor)| PrefixedKeyword::new(prefix, constructor))
                    .collect();
                for special_case in special_cases {
                    if special_case.handles(input) {
                        return special_case.transform(input);
                    }
                }
                Ok(Keyword::Unrecognized(input.into()))
                //Err(ParseKeywordError::UnknownKeyword)
            }
        }
    }
}

trait KeywordSpecialCase {
    fn handles(&self, input: &str) -> bool;
    fn transform(&self, input: &str) -> Result<Keyword, ParseKeywordError>;
}

struct PrefixedKeyword<'a> {
    prefix: &'a str,
    constructor: &'a (dyn Fn(u16) -> Keyword),
}

impl<'a> PrefixedKeyword<'a> {
    fn new(prefix: &'a str, constructor: &'a (dyn Fn(u16) -> Keyword)) -> PrefixedKeyword<'a> {
        PrefixedKeyword {
            prefix,
            constructor,
        }
    }
}

impl<'a> KeywordSpecialCase for PrefixedKeyword<'a> {
    fn handles(&self, input: &str) -> bool {
        input.starts_with(self.prefix)
    }

    fn transform(&self, input: &str) -> Result<Keyword, ParseKeywordError> {
        let (_, representation) = input.split_at(self.prefix.len());
        match u16::from_str(representation) {
            Ok(n) => Ok((self.constructor)(n)),
            Err(_) => Err(ParseKeywordError::NotANumber),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_could_be_constructed_from_str() {
        let data = vec![
            ("AV", Keyword::AV),
            ("BITPIX", Keyword::BITPIX),
            ("CAMPAIGN", Keyword::CAMPAIGN),
            ("CHANNEL", Keyword::CHANNEL),
            ("CHECKSUM", Keyword::CHECKSUM),
            ("COMMENT", Keyword::COMMENT),
            ("CREATOR", Keyword::CREATOR),
            ("DATASUM", Keyword::DATASUM),
            ("DATA_REL", Keyword::DATA_REL),
            ("DATE", Keyword::DATE),
            ("DEC_OBJ", Keyword::DEC_OBJ),
            ("EBMINUSV", Keyword::EBMINUSV),
            ("END", Keyword::END),
            ("EQUINOX", Keyword::EQUINOX),
            ("EXTEND", Keyword::EXTEND),
            ("EXTVER", Keyword::EXTVER),
            ("FEH", Keyword::FEH),
            ("FILEVER", Keyword::FILEVER),
            ("GCOUNT", Keyword::GCOUNT),
            ("GKCOLOR", Keyword::GKCOLOR),
            ("GLAT", Keyword::GLAT),
            ("GLON", Keyword::GLON),
            ("GMAG", Keyword::GMAG),
            ("GRCOLOR", Keyword::GRCOLOR),
            ("HISTORY", Keyword::HISTORY),
            ("HMAG", Keyword::HMAG),
            ("IMAG", Keyword::IMAG),
            ("INSTRUME", Keyword::INSTRUME),
            ("JKCOLOR", Keyword::JKCOLOR),
            ("JMAG", Keyword::JMAG),
            ("KEPLERID", Keyword::KEPLERID),
            ("KEPMAG", Keyword::KEPMAG),
            ("KMAG", Keyword::KMAG),
            ("LOGG", Keyword::LOGG),
            ("MISSION", Keyword::MISSION),
            ("MODULE", Keyword::MODULE),
            ("NAXIS", Keyword::NAXIS),
            ("NEXTEND", Keyword::NEXTEND),
            ("OBJECT", Keyword::OBJECT),
            ("OBSMODE", Keyword::OBSMODE),
            ("ORIGIN", Keyword::ORIGIN),
            ("OUTPUT", Keyword::OUTPUT),
            ("PARALLAX", Keyword::PARALLAX),
            ("PCOUNT", Keyword::PCOUNT),
            ("PMDEC", Keyword::PMDEC),
            ("PMRA", Keyword::PMRA),
            ("PMTOTAL", Keyword::PMTOTAL),
            ("PROCVER", Keyword::PROCVER),
            ("RADESYS", Keyword::RADESYS),
            ("RADIUS", Keyword::RADIUS),
            ("RA_OBJ", Keyword::RA_OBJ),
            ("RMAG", Keyword::RMAG),
            ("SIMPLE", Keyword::SIMPLE),
            ("TEFF", Keyword::TEFF),
            ("TELESCOP", Keyword::TELESCOP),
            ("TFIELDS", Keyword::TFIELDS),
            ("TIMVERSN", Keyword::TIMVERSN),
            ("THEAP", Keyword::THEAP),
            ("TMINDEX", Keyword::TMINDEX),
            ("TTABLEID", Keyword::TTABLEID),
            ("XTENSION", Keyword::XTENSION),
            ("ZMAG", Keyword::ZMAG),
        ];

        for (input, expected) in data {
            assert_eq!(Keyword::from_str(input).unwrap(), expected);
        }
    }

    #[allow(non_snake_case)]
    #[test]
    fn TDIMn_should_be_parsed_from_str() {
        for n in 1u16..1000u16 {
            let keyword = Keyword::TDIMn(n);
            let representation = format!("TDIM{}", n);

            assert_eq!(Keyword::from_str(&representation).unwrap(), keyword);
        }
    }

    #[allow(non_snake_case)]
    #[test]
    fn TDISPn_should_be_parsed_from_str() {
        for n in 1u16..1000u16 {
            let keyword = Keyword::TDISPn(n);
            let representation = format!("TDISP{}", n);

            assert_eq!(Keyword::from_str(&representation).unwrap(), keyword);
        }
    }

    #[allow(non_snake_case)]
    #[test]
    fn NAXISn_should_be_parsed_from_str() {
        for n in 1u16..1000u16 {
            let keyword = Keyword::NAXISn(n);
            let representation = format!("NAXIS{}", n);

            assert_eq!(Keyword::from_str(&representation).unwrap(), keyword);
        }
    }

    #[allow(non_snake_case)]
    #[test]
    fn TFORM_should_be_parsed_from_str() {
        for n in 1u16..1000u16 {
            let keyword = Keyword::TFORMn(n);
            let representation = format!("TFORM{}", n);

            assert_eq!(Keyword::from_str(&representation).unwrap(), keyword);
        }
    }

    #[allow(non_snake_case)]
    #[test]
    fn TTYPE_should_be_parsed_from_str() {
        for n in 1u16..1000u16 {
            let keyword = Keyword::TTYPEn(n);
            let representation = format!("TTYPE{}", n);

            assert_eq!(Keyword::from_str(&representation).unwrap(), keyword);
        }
    }

    #[allow(non_snake_case)]
    #[test]
    fn TSCALn_should_be_parsed_from_str() {
        for n in 1u16..1000u16 {
            let keyword = Keyword::TSCALn(n);
            let representation = format!("TSCAL{}", n);

            assert_eq!(Keyword::from_str(&representation).unwrap(), keyword);
        }
    }

    #[allow(non_snake_case)]
    #[test]
    fn TZEROn_should_be_parsed_from_str() {
        for n in 1u16..1000u16 {
            let keyword = Keyword::TZEROn(n);
            let representation = format!("TZERO{}", n);

            assert_eq!(Keyword::from_str(&representation).unwrap(), keyword);
        }
    }

    #[allow(non_snake_case)]
    #[test]
    fn TNULL_should_be_parsed_from_str() {
        for n in 1u16..1000u16 {
            let keyword = Keyword::TNULLn(n);
            let representation = format!("TNULL{}", n);

            assert_eq!(Keyword::from_str(&representation).unwrap(), keyword);
        }
    }

    #[allow(non_snake_case)]
    #[test]
    fn TUNIT_should_be_parsed_from_str() {
        for n in 1u16..1000u16 {
            let keyword = Keyword::TUNITn(n);
            let representation = format!("TUNIT{}", n);

            assert_eq!(Keyword::from_str(&representation).unwrap(), keyword);
        }
    }

    #[test]
    fn should_also_parse_whitespace_keywords() {
        assert_eq!(Keyword::from_str("SIMPLE  ").unwrap(), Keyword::SIMPLE);
    }
}
