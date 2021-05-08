//! Input/Output for CITI records
//! 
//! <p><a href="http://literature.cdn.keysight.com/litweb/pdf/ads15/cktsim/ck2016.html#:~:text=CITIrecord%20stands%20for%20Common%20Instrumentation,it%20can%20meet%20future%20needs">The standard</a>, defines the following entities:</p>
//! 
//! | Name     | Description                     |
//! |----------|---------------------------------|
//! | Record   | The entire contents of the record |
//! | Header   | Header of the record              |
//! | Data     | One or more data arrays         |
//! | Keywords | Define the header contents      |
//! 
//! As this is a custom ASCII record type, the standard is not as simple as one would like.
//! The standard is followed as closely as is reasonable. The largest changes are in the
//! extension of the keywords.
//! 
//! ## Non-Standard Type
//! 
//! A non-standard but industry prevelent comment section is added formated with a bang:
//! 
//! ```.no_test
//! !COMMENT
//! ```
//! 
//! These are used to provide internal comments.
//! 
//! ## Input-Output consistency:
//! 
//! General input-output consistency cannot be guaranteed with CITI records because of their design.
//! That is, if a record is read in and read out, the byte representation of the record may change,
//! exact floating point representations may change, but the record will contain the same information.
//! The following is not guaranteed:
//! 
//! - ASCII representation of floating points may change because of the String -> Float -> String conversion.
//! - Floats may be shifted in exponential format.
//! - All `SEG_LIST` keywords will be converted to `VAR_LIST`

use lazy_static::lazy_static;
use regex::Regex;

use std::convert::TryFrom;
use std::str::FromStr;
use std::fmt;
use std::path::{Path,PathBuf};
use std::io::Write;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Parsing error: `{0}`")]
    ParseError(#[from] ParseError),
    #[error("Reading error: `{0}`")]
    ReaderError(#[from] ReaderError),
    #[error("Invalid record error: `{0}`")]
    ValidRecordError(#[from] ValidRecordError),
    #[error("Error writing record: `{0}`")]
    WriteError(#[from] WriteError)
}
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod test_error {
    use super::*;

    mod test_display {
        use super::*;

        #[test]
        fn parse_error() {
            let error = Error::ParseError(ParseError::BadRegex);
            assert_eq!(format!("{}", error), "Parsing error: `Regex could not be parsed`");
        }

        #[test]
        fn reader_error() {
            let error = Error::ReaderError(ReaderError::DataArrayOverIndex);
            assert_eq!(format!("{}", error), "Reading error: `More data arrays than defined in header`");
        }

        #[test]
        fn valid_record_error() {
            let error = Error::ValidRecordError(ValidRecordError::NoVersion);
            assert_eq!(format!("{}", error), "Invalid record error: `Version is not defined`");
        }

        #[test]
        fn write_error() {
            let error = Error::WriteError(WriteError::NoVersion);
            assert_eq!(format!("{}", error), "Error writing record: `Version is not defined`");
        }
    }

    mod from_error {
        use super::*;

        #[test]
        fn from_parse_error() {
            match Error::from(ParseError::BadRegex) {
                Error::ParseError(ParseError::BadRegex) => (),
                _ => panic!("Incorrect from parse error"),
            }
        }

        #[test]
        fn from_reader_error() {
            match Error::from(ReaderError::DataArrayOverIndex) {
                Error::ReaderError(ReaderError::DataArrayOverIndex) => (),
                _ => panic!("Incorrect from reader error"),
            }
        }

        #[test]
        fn from_valid_record_error() {
            match Error::from(ValidRecordError::NoName) {
                Error::ValidRecordError(ValidRecordError::NoName) => (),
                _ => panic!("Incorrect from valid error"),
            }
        }

        #[test]
        fn from_write_error() {
            match Error::from(WriteError::NoVersion) {
                Error::WriteError(WriteError::NoVersion) => (),
                _ => panic!("Incorrect from write error"),
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Keyword `{0}` is not supported")]
    BadKeyword(String),
    #[error("Regex could not be parsed")]
    BadRegex,
    #[error("Cannot parse as number `{0}`")]
    NumberParseError(String),
}
// type ParseResult<T> = std::result::Result<T, ParseError>;

#[cfg(test)]
mod test_parse_error {
    use super::*;

    mod test_display {
        use super::*;

        #[test]
        fn bad_keyword() {
            let error = ParseError::BadKeyword(String::from("asdf"));
            assert_eq!(format!("{}", error), "Keyword `asdf` is not supported");
        }

        #[test]
        fn bad_keyword_second() {
            let error = ParseError::BadKeyword(String::from("----"));
            assert_eq!(format!("{}", error), "Keyword `----` is not supported");
        }

        #[test]
        fn number_parse_error() {
            let error = ParseError::NumberParseError(String::from("asdf"));
            assert_eq!(format!("{}", error), "Cannot parse as number `asdf`");
        }

        #[test]
        fn number_parse_error_second() {
            let error = ParseError::NumberParseError(String::from("----"));
            assert_eq!(format!("{}", error), "Cannot parse as number `----`");
        }

        #[test]
        fn bad_regex() {
            let error = ParseError::BadRegex;
            assert_eq!(format!("{}", error), "Regex could not be parsed");
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Keywords {
    /// CITIFile version e.g. A.01.01
    CITIFile{version: String},
    /// Name. Single word with no spaces. e.g. CAL_SET
    Name(String),
    /// Independent variable with name, optional format, and number of samples. e.g. VAR FREQ MAG 201
    Var{name: String, format: Option<String>, length: usize},
    /// Constant with name and value. e.g. CONSTANT A A_THING
    Constant{name: String, value: String},
    /// New Device
    Device{name: String, value: String},
    /// Beginning of independent variable segments
    SegListBegin,
    /// An item in a SEG list
    SegItem{first: f64, last: f64, number: usize},
    /// End of independent variable segments
    SegListEnd,
    /// Beginning of independent variable list
    VarListBegin,
    /// Item in a var list
    VarListItem(f64),
    /// End of independent variable list
    VarListEnd,
    /// Define a data array. e.g. DATA S\[1,1\] RI
    Data{name: String, format: String},
    /// Real, Imaginary pair in data
    DataPair{real: f64, imag: f64},
    /// Begin data array
    Begin,
    /// End data array
    End,
    /// Comment (non-standard)
    Comment(String),
}

impl FromStr for Keywords {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Keywords::try_from(s)
    }
}

impl TryFrom<&str> for Keywords {
    type Error = ParseError;

    fn try_from(line: &str) -> std::result::Result<Self, Self::Error> {
        // Avoid recompiling each time
        lazy_static! {
            static ref RE_DEVICE: Regex = Regex::new(r"^#(?P<Name>\S+) (?P<Value>.*)$").unwrap();
            static ref RE_VAR: Regex = Regex::new(r"^VAR (?P<Name>\S+) ?(?P<Format>\S*) (?P<Length>\d+)$").unwrap();
            static ref RE_CITIFILE: Regex = Regex::new(r"^CITIFILE (?P<Version>\S+)$").unwrap();
            static ref RE_NAME: Regex = Regex::new(r"^NAME (?P<Name>\S+)$").unwrap();
            static ref RE_DATA: Regex = Regex::new(r"^DATA (?P<Name>\S+) (?P<Format>\S+)$").unwrap();
            static ref RE_SEG_ITEM: Regex = Regex::new(r"^SEG (?P<First>[+-]?(\d+)\.?\d*[eE]?[+-]?\d+) (?P<Last>[+-]?(\d+)\.?\d*[eE]?[+-]?\d+) (?P<Number>\d+)$").unwrap();
            static ref RE_VAR_ITEM: Regex = Regex::new(r"^(?P<Value>[+-]?(\d+)\.?\d*[eE]?[+-]?\d+)$").unwrap();
            static ref RE_DATA_PAIR: Regex = Regex::new(r"^(?P<Real>\S+),\s*(?P<Imag>\S+)$").unwrap();
            static ref RE_CONSTANT: Regex = Regex::new(r"^CONSTANT (?P<Name>\S+) (?P<Value>\S+)$").unwrap();
            static ref RE_COMMENT: Regex = Regex::new(r"^!(?P<Comment>.*)$").unwrap();
        }

        match line {
            "SEG_LIST_BEGIN" => Ok(Keywords::SegListBegin),
            "SEG_LIST_END" => Ok(Keywords::SegListEnd),
            "VAR_LIST_BEGIN" => Ok(Keywords::VarListBegin),
            "VAR_LIST_END" => Ok(Keywords::VarListEnd),
            "BEGIN" => Ok(Keywords::Begin),
            "END" => Ok(Keywords::End),
            _ if RE_DATA_PAIR.is_match(line) => {
                let cap = RE_DATA_PAIR.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keywords::DataPair{
                    real: cap.name("Real").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?.parse::<f64>().map_err(|_| ParseError::NumberParseError(String::from(line)))?,
                    imag: cap.name("Imag").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?.parse::<f64>().map_err(|_| ParseError::NumberParseError(String::from(line)))?,
                })
            },
            _ if RE_DEVICE.is_match(line) => {
                let cap = RE_DEVICE.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keywords::Device{
                    name: String::from(cap.name("Name").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?),
                    value: String::from(cap.name("Value").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?),
                })
            },
            _ if RE_SEG_ITEM.is_match(line) => {
                let cap = RE_SEG_ITEM.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keywords::SegItem{
                    first: cap.name("First").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?.parse::<f64>().map_err(|_| ParseError::NumberParseError(String::from(line)))?,
                    last: cap.name("Last").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?.parse::<f64>().map_err(|_| ParseError::NumberParseError(String::from(line)))?,
                    number: cap.name("Number").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?.parse::<usize>().map_err(|_| ParseError::NumberParseError(String::from(line)))?,
                })
            },
            _ if RE_VAR_ITEM.is_match(line) => {
                let cap = RE_VAR_ITEM.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keywords::VarListItem(
                    cap.name("Value").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?.parse::<f64>().map_err(|_| ParseError::NumberParseError(String::from(line)))?
                ))
            },
            _ if RE_DATA.is_match(line) => {
                let cap = RE_DATA.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keywords::Data{
                    name: String::from(cap.name("Name").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?),
                    format: String::from(cap.name("Format").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?),
                })
            },
            _ if RE_VAR.is_match(line) => {
                let cap = RE_VAR.captures(line).ok_or(ParseError::BadRegex)?;
                let closure = |m: String| {if m.is_empty() {None} else {Some(m)}};
                Ok(Keywords::Var{
                    name: String::from(cap.name("Name").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?),
                    format: closure(cap.name("Format").map(|m| String::from(m.as_str())).ok_or(ParseError::BadRegex)?),
                    length: cap.name("Length").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?.parse::<usize>().map_err(|_| ParseError::NumberParseError(String::from(line)))?,
                })
            },
            _ if RE_COMMENT.is_match(line) => {
                let cap = RE_COMMENT.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keywords::Comment(String::from(cap.name("Comment").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?)))
            },
            _ if RE_CITIFILE.is_match(line) => {
                let cap = RE_CITIFILE.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keywords::CITIFile{
                    version: String::from(cap.name("Version").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?)
                })
            },
            _ if RE_NAME.is_match(line) => {
                let cap = RE_NAME.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keywords::Name(String::from(cap.name("Name").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?)))
            },
            _ if RE_CONSTANT.is_match(line) => {
                let cap = RE_CONSTANT.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keywords::Constant{
                    name: String::from(cap.name("Name").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?),
                    value: String::from(cap.name("Value").map(|m| m.as_str()).ok_or(ParseError::BadRegex)?)
                })
            },
            _ => Err(ParseError::BadKeyword(String::from(line))),
        }
    }
}

impl fmt::Display for Keywords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Keywords::CITIFile{version} => write!(f, "CITIFILE {}", version),
            Keywords::Name(name) => write!(f, "NAME {}", name),
            Keywords::Var{name, format, length} => match format {
                Some(form) => write!(f, "VAR {} {} {}", name, form, length),
                None => write!(f, "VAR {} {}", name, length),
            },
            Keywords::Constant{name, value} => write!(f, "CONSTANT {} {}", name, value),
            Keywords::Device{name, value} => write!(f, "#{} {}", name, value),
            Keywords::SegListBegin => write!(f, "SEG_LIST_BEGIN"),
            Keywords::SegItem{first, last, number} => write!(f, "SEG {} {} {}", first, last, number),
            Keywords::SegListEnd => write!(f, "SEG_LIST_END"),
            Keywords::VarListBegin => write!(f, "VAR_LIST_BEGIN"),
            Keywords::VarListItem(n) => write!(f, "{}", n),
            Keywords::VarListEnd => write!(f, "VAR_LIST_END"),
            Keywords::Data{name, format} => write!(f, "DATA {} {}", name, format),
            Keywords::DataPair{real, imag} => write!(f, "{:E},{:E}", real, imag),
            Keywords::Begin => write!(f, "BEGIN"),
            Keywords::End => write!(f, "END"),
            Keywords::Comment(comment) => write!(f, "!{}", comment),
        }
    }
}

#[cfg(test)]
mod test_keywords {
    use super::*;

    #[cfg(test)]
    mod test_fmt_display {
        use super::*;

        #[test]
        fn citirecord_a_01_00() {
            let keyword = Keywords::CITIFile{version: String::from("A.01.00")};
            assert_eq!("CITIFILE A.01.00", format!("{}", keyword));
        }

        #[test]
        fn citirecord_a_01_01() {
            let keyword = Keywords::CITIFile{version: String::from("A.01.01")};
            assert_eq!("CITIFILE A.01.01", format!("{}", keyword));
        }

        #[test]
        fn name() {
            let keyword = Keywords::Name(String::from("CAL_SET"));
            assert_eq!("NAME CAL_SET", format!("{}", keyword));
        }

        #[test]
        fn var_standard() {
            let keyword = Keywords::Var{name: String::from("FREQ"), format: Some(String::from("MAG")), length: 201};
            assert_eq!("VAR FREQ MAG 201", format!("{}", keyword));
        }

        #[test]
        fn var_no_format() {
            let keyword = Keywords::Var{name: String::from("FREQ"), format: None, length: 202};
            assert_eq!("VAR FREQ 202", format!("{}", keyword));
        }

        #[test]
        fn constant() {
            let keyword = Keywords::Constant{name: String::from("A_CONSTANT"), value: String::from("1.2345")};
            assert_eq!("CONSTANT A_CONSTANT 1.2345", format!("{}", keyword));
        }

        #[test]
        fn device() {
            let keyword = Keywords::Device{name: String::from("NA"), value: String::from("REGISTER 1")};
            assert_eq!("#NA REGISTER 1", format!("{}", keyword));
        }

        #[test]
        fn device_number() {
            let keyword = Keywords::Device{name: String::from("NA"), value: String::from("POWER2 1.0E1")};
            assert_eq!("#NA POWER2 1.0E1", format!("{}", keyword));
        }

        #[test]
        fn device_name() {
            let keyword = Keywords::Device{name: String::from("WVI"), value: String::from("A B")};
            assert_eq!("#WVI A B", format!("{}", keyword));
        }

        #[test]
        fn seg_list_begin() {
            let keyword = Keywords::SegListBegin;
            assert_eq!("SEG_LIST_BEGIN", format!("{}", keyword));
        }

        #[test]
        fn seg_item() {
            let keyword = Keywords::SegItem{first: 1000000000., last: 4000000000., number: 10};
            assert_eq!("SEG 1000000000 4000000000 10", format!("{}", keyword));
        }

        #[test]
        fn seg_list_end() {
            let keyword = Keywords::SegListEnd;
            assert_eq!("SEG_LIST_END", format!("{}", keyword));
        }

        #[test]
        fn var_list_begin() {
            let keyword = Keywords::VarListBegin;
            assert_eq!("VAR_LIST_BEGIN", format!("{}", keyword));
        }

        #[test]
        fn var_item() {
            let keyword = Keywords::VarListItem(100000.);
            assert_eq!("100000", format!("{}", keyword));
        }

        #[test]
        fn var_list_end() {
            let keyword = Keywords::VarListEnd;
            assert_eq!("VAR_LIST_END", format!("{}", keyword));
        }

        #[test]
        fn data_s11() {
            let keyword = Keywords::Data{name: String::from("S[1,1]"), format: String::from("RI")};
            assert_eq!("DATA S[1,1] RI", format!("{}", keyword));
        }

        #[test]
        fn data_e() {
            let keyword = Keywords::Data{name: String::from("E"), format: String::from("RI")};
            assert_eq!("DATA E RI", format!("{}", keyword));
        }

        #[test]
        fn data_pair_simple() {
            let keyword = Keywords::DataPair{real: 1e9, imag: -1e9};
            assert_eq!("1E9,-1E9", format!("{}", keyword));
        }
        
        #[test]
        fn data_pair() {
            let keyword = Keywords::DataPair{real: 0.86303e-1, imag: -8.98651e-1};
            assert_eq!("8.6303E-2,-8.98651E-1", format!("{}", keyword));
        }

        #[test]
        fn begin() {
            let keyword = Keywords::Begin;
            assert_eq!("BEGIN", format!("{}", keyword));
        }

        #[test]
        fn end() {
            let keyword = Keywords::End;
            assert_eq!("END", format!("{}", keyword));
        }

        #[test]
        fn comment() {
            let keyword = Keywords::Comment(String::from("DATE: 2019.11.01"));
            assert_eq!("!DATE: 2019.11.01", format!("{}", keyword));
        }
    }

    #[cfg(test)]
    mod test_from_str_slice {
        use super::*;
        use approx::*;

        #[test]
        fn fails_on_bad_string() {
            match Keywords::from_str("this is a bad string") {
                Err(ParseError::BadKeyword(bad_string)) => assert_eq!(bad_string, "this is a bad string"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn citirecord_a_01_00() {
            match Keywords::from_str("CITIFILE A.01.00") {
                Ok(Keywords::CITIFile{version}) => assert_eq!(version, "A.01.00"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn citirecord_a_01_01() {
            match Keywords::from_str("CITIFILE A.01.01") {
                Ok(Keywords::CITIFile{version}) => assert_eq!(version, "A.01.01"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn name_cal_set() {
            match Keywords::from_str("NAME CAL_SET") {
                Ok(Keywords::Name(name)) => assert_eq!(name, "CAL_SET"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn name_raw_data() {
            match Keywords::from_str("NAME RAW_DATA") {
                Ok(Keywords::Name(name)) => assert_eq!(name, "RAW_DATA"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn constant() {
            match Keywords::from_str("CONSTANT A_CONSTANT 1.2345") {
                Ok(Keywords::Constant{name, value}) => {
                    assert_eq!(name, "A_CONSTANT");
                    assert_eq!(value, "1.2345");
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn device() {
            match Keywords::from_str("#NA REGISTER 1") {
                Ok(Keywords::Device{name, value}) => {
                    assert_eq!(name, "NA");
                    assert_eq!(value, "REGISTER 1");
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn device_number() {
            match Keywords::from_str("#NA POWER2 1.0E1") {
                Ok(Keywords::Device{name, value}) => {
                    assert_eq!(name, "NA");
                    assert_eq!(value, "POWER2 1.0E1");
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn device_name() {
            match Keywords::from_str("#WVI A B") {
                Ok(Keywords::Device{name, value}) => {
                    assert_eq!(name, "WVI");
                    assert_eq!(value, "A B");
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_standard() {
            match Keywords::from_str("VAR FREQ MAG 201") {
                Ok(Keywords::Var{name, format, length}) => {
                    assert_eq!(name, "FREQ");
                    assert_eq!(format, Some(String::from("MAG")));
                    assert_eq!(length, 201);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_no_format() {
            match Keywords::from_str("VAR FREQ 202") {
                Ok(Keywords::Var{name, format, length}) => {
                    assert_eq!(name, "FREQ");
                    assert_eq!(format, None);
                    assert_eq!(length, 202);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_list_begin() {
            match Keywords::from_str("SEG_LIST_BEGIN") {
                Ok(Keywords::SegListBegin) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_item() {
            match Keywords::from_str("SEG 1000000000 4000000000 10") {
                Ok(Keywords::SegItem{first, last, number}) => {
                    assert_relative_eq!(first, 1000000000.);
                    assert_relative_eq!(last, 4000000000.);
                    assert_eq!(number, 10);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_item_exponential() {
            match Keywords::from_str("SEG 1e9 1E4 100") {
                Ok(Keywords::SegItem{first, last, number}) => {
                    assert_relative_eq!(first, 1e9);
                    assert_relative_eq!(last, 1e4);
                    assert_eq!(number, 100);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_item_negative() {
            match Keywords::from_str("SEG -1e9 1E-4 1") {
                Ok(Keywords::SegItem{first, last, number}) => {
                    assert_relative_eq!(first, -1e9);
                    assert_relative_eq!(last, 1e-4);
                    assert_eq!(number, 1);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_list_end() {
            match Keywords::from_str("SEG_LIST_END") {
                Ok(Keywords::SegListEnd) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_list_begin() {
            match Keywords::from_str("VAR_LIST_BEGIN") {
                Ok(Keywords::VarListBegin) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item() {
            match Keywords::from_str("100000") {
                Ok(Keywords::VarListItem(value)) => {
                    assert_relative_eq!(value, 100000.);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item_exponential() {
            match Keywords::from_str("100E+6") {
                Ok(Keywords::VarListItem(value)) => {
                    assert_relative_eq!(value, 100E+6);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item_negative_exponential() {
            match Keywords::from_str("-1e-2") {
                Ok(Keywords::VarListItem(value)) => {
                    assert_relative_eq!(value, -1e-2);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item_negative() {
            match Keywords::from_str("-100000") {
                Ok(Keywords::VarListItem(value)) => {
                    assert_relative_eq!(value, -100000.);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_list_end() {
            match Keywords::from_str("VAR_LIST_END") {
                Ok(Keywords::VarListEnd) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_s11() {
            match Keywords::from_str("DATA S[1,1] RI") {
                Ok(Keywords::Data{name, format}) => {
                    assert_eq!(name, "S[1,1]");
                    assert_eq!(format, "RI");
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_e() {
            match Keywords::from_str("DATA E RI") {
                Ok(Keywords::Data{name, format}) => {
                    assert_eq!(name, "E");
                    assert_eq!(format, "RI");
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_pair_simple() {
            match Keywords::from_str("1E9,-1E9") {
                Ok(Keywords::DataPair{real, imag}) => {
                    assert_relative_eq!(real, 1e9);
                    assert_relative_eq!(imag, -1e9);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_pair() {
            match Keywords::from_str("8.6303E-2,-8.98651E-1") {
                Ok(Keywords::DataPair{real, imag}) => {
                    assert_relative_eq!(real, 0.86303e-1);
                    assert_relative_eq!(imag, -8.98651e-1);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_pair_spaced() {
            match Keywords::from_str("8.6303E-2, -8.98651E-1") {
                Ok(Keywords::DataPair{real, imag}) => {
                    assert_relative_eq!(real, 0.86303e-1);
                    assert_relative_eq!(imag, -8.98651e-1);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn begin() {
            match Keywords::from_str("BEGIN") {
                Ok(Keywords::Begin) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn end() {
            match Keywords::from_str("END") {
                Ok(Keywords::End) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn comment() {
            match Keywords::from_str("!DATE: 2019.11.01") {
                Ok(Keywords::Comment(s)) => assert_eq!(s, "DATE: 2019.11.01"),
                e => panic!("{:?}", e),
            }
        }
    }

    #[cfg(test)]
    mod test_try_from_string_slice {
        use super::*;
        use approx::*;

        #[test]
        fn fails_on_bad_string() {
            match Keywords::try_from("this is a bad string") {
                Err(ParseError::BadKeyword(bad_string)) => assert_eq!(bad_string, "this is a bad string"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn citirecord_a_01_00() {
            match Keywords::try_from("CITIFILE A.01.00") {
                Ok(Keywords::CITIFile{version}) => assert_eq!(version, "A.01.00"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn citirecord_a_01_01() {
            match Keywords::try_from("CITIFILE A.01.01") {
                Ok(Keywords::CITIFile{version}) => assert_eq!(version, "A.01.01"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn name_cal_set() {
            match Keywords::try_from("NAME CAL_SET") {
                Ok(Keywords::Name(name)) => assert_eq!(name, "CAL_SET"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn name_raw_data() {
            match Keywords::try_from("NAME RAW_DATA") {
                Ok(Keywords::Name(name)) => assert_eq!(name, "RAW_DATA"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn constant() {
            match Keywords::try_from("CONSTANT A_CONSTANT 1.2345") {
                Ok(Keywords::Constant{name, value}) => {
                    assert_eq!(name, "A_CONSTANT");
                    assert_eq!(value, "1.2345");
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn device() {
            match Keywords::try_from("#NA REGISTER 1") {
                Ok(Keywords::Device{name, value}) => {
                    assert_eq!(name, "NA");
                    assert_eq!(value, "REGISTER 1");
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn device_number() {
            match Keywords::try_from("#NA POWER2 1.0E1") {
                Ok(Keywords::Device{name, value}) => {
                    assert_eq!(name, "NA");
                    assert_eq!(value, "POWER2 1.0E1");
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn device_name() {
            match Keywords::try_from("#WVI A B") {
                Ok(Keywords::Device{name, value}) => {
                    assert_eq!(name, "WVI");
                    assert_eq!(value, "A B");
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_standard() {
            match Keywords::try_from("VAR FREQ MAG 201") {
                Ok(Keywords::Var{name, format, length}) => {
                    assert_eq!(name, "FREQ");
                    assert_eq!(format, Some(String::from("MAG")));
                    assert_eq!(length, 201);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_no_format() {
            match Keywords::try_from("VAR FREQ 202") {
                Ok(Keywords::Var{name, format, length}) => {
                    assert_eq!(name, "FREQ");
                    assert_eq!(format, None);
                    assert_eq!(length, 202);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_list_begin() {
            match Keywords::try_from("SEG_LIST_BEGIN") {
                Ok(Keywords::SegListBegin) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_item() {
            match Keywords::try_from("SEG 1000000000 4000000000 10") {
                Ok(Keywords::SegItem{first, last, number}) => {
                    assert_relative_eq!(first, 1000000000.);
                    assert_relative_eq!(last, 4000000000.);
                    assert_eq!(number, 10);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_item_exponential() {
            match Keywords::try_from("SEG 1e9 1E4 100") {
                Ok(Keywords::SegItem{first, last, number}) => {
                    assert_relative_eq!(first, 1e9);
                    assert_relative_eq!(last, 1e4);
                    assert_eq!(number, 100);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_item_negative() {
            match Keywords::try_from("SEG -1e9 1E-4 1") {
                Ok(Keywords::SegItem{first, last, number}) => {
                    assert_relative_eq!(first, -1e9);
                    assert_relative_eq!(last, 1e-4);
                    assert_eq!(number, 1);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_list_end() {
            match Keywords::try_from("SEG_LIST_END") {
                Ok(Keywords::SegListEnd) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_list_begin() {
            match Keywords::try_from("VAR_LIST_BEGIN") {
                Ok(Keywords::VarListBegin) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item() {
            match Keywords::try_from("100000") {
                Ok(Keywords::VarListItem(value)) => {
                    assert_relative_eq!(value, 100000.);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item_exponential() {
            match Keywords::try_from("100E+6") {
                Ok(Keywords::VarListItem(value)) => {
                    assert_relative_eq!(value, 100E+6);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item_negative_exponential() {
            match Keywords::try_from("-1e-2") {
                Ok(Keywords::VarListItem(value)) => {
                    assert_relative_eq!(value, -1e-2);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item_negative() {
            match Keywords::try_from("-100000") {
                Ok(Keywords::VarListItem(value)) => {
                    assert_relative_eq!(value, -100000.);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_list_end() {
            match Keywords::try_from("VAR_LIST_END") {
                Ok(Keywords::VarListEnd) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_s11() {
            match Keywords::try_from("DATA S[1,1] RI") {
                Ok(Keywords::Data{name, format}) => {
                    assert_eq!(name, "S[1,1]");
                    assert_eq!(format, "RI");
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_e() {
            match Keywords::try_from("DATA E RI") {
                Ok(Keywords::Data{name, format}) => {
                    assert_eq!(name, "E");
                    assert_eq!(format, "RI");
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_pair_simple() {
            match Keywords::try_from("1E9,-1E9") {
                Ok(Keywords::DataPair{real, imag}) => {
                    assert_relative_eq!(real, 1e9);
                    assert_relative_eq!(imag, -1e9);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_pair() {
            match Keywords::try_from("8.6303E-2,-8.98651E-1") {
                Ok(Keywords::DataPair{real, imag}) => {
                    assert_relative_eq!(real, 0.86303e-1);
                    assert_relative_eq!(imag, -8.98651e-1);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_pair_spaced() {
            match Keywords::try_from("8.6303E-2, -8.98651E-1") {
                Ok(Keywords::DataPair{real, imag}) => {
                    assert_relative_eq!(real, 0.86303e-1);
                    assert_relative_eq!(imag, -8.98651e-1);
                },
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn begin() {
            match Keywords::try_from("BEGIN") {
                Ok(Keywords::Begin) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn end() {
            match Keywords::try_from("END") {
                Ok(Keywords::End) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn comment() {
            match Keywords::try_from("!DATE: 2019.11.01") {
                Ok(Keywords::Comment(s)) => assert_eq!(s, "DATE: 2019.11.01"),
                e => panic!("{:?}", e),
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Device {
    pub name: String,
    pub entries: Vec<String>,
}

impl Device {
    pub fn new(name: &str) -> Device {
        Device {
            name: String::from(name),
            entries: vec![],
        }
    }
}

#[cfg(test)]
mod test_device {
    use super::*;

    #[test]
    fn test_new() {
        let result = Device::new("A Name");
        let expected = Device {name: String::from("A Name"), entries: vec![]};
        assert_eq!(result, expected);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Var {
    pub name: Option<String>,
    pub format: Option<String>,
    pub data: Vec<f64>,
}

impl Var {
    fn blank() -> Var {
        Var {
            name: None,
            format: None,
            data: vec![],
        }
    }

    pub fn new(name: &str, format: Option<String>) -> Var {
        Var {
            name: Some(String::from(name)),
            format: format,
            data: vec![],
        }
    }

    pub fn push(&mut self, value: f64) {
        self.data.push(value);
    }

    pub fn seq(&mut self, first: f64, last: f64, number: usize) {
        match number {
            0 => (),
            1 => self.push(first),
            _ => {
                let delta = (last - first) / ((number-1) as f64);
                for i in 0..number {
                    self.push(first + (i as f64)*delta);
                }
            },
        }
    }
}

#[cfg(test)]
mod test_var {
    use super::*;

    #[test]
    fn test_blank() {
        let result = Var::blank();
        let expected = Var {name: None, format: None, data: vec![]};
        assert_eq!(result, expected);
    }

    #[test]
    fn test_new() {
        let result = Var::new("Name", None);
        let expected = Var {name: Some(String::from("Name")), format: None, data: vec![]};
        assert_eq!(result, expected);
    }

    mod test_push {
        use super::*;

        #[test]
        fn empty() {
            let mut var = Var {name: None, format: None, data: vec![]};
            var.push(1.);
            assert_eq!(vec![1.], var.data);
        }

        #[test]
        fn double() {
            let mut var = Var {name: None, format: None, data: vec![]};
            var.push(1.);
            var.push(2.);
            assert_eq!(vec![1., 2.], var.data);
        }

        #[test]
        fn existing() {
            let mut var = Var {name: None, format: None, data: vec![1.]};
            var.push(2.);
            assert_eq!(vec![1., 2.], var.data);
        }
    }

    mod test_seq {
        use super::*;

        #[test]
        fn number_zero() {
            let mut var = Var {name: None, format: None, data: vec![]};
            var.seq(1., 2., 0);
            assert_eq!(Vec::<f64>::new(), var.data);
        }

        #[test]
        fn number_one() {
            let mut var = Var {name: None, format: None, data: vec![]};
            var.seq(10., 20., 1);
            assert_eq!(vec![10.], var.data);
        }

        #[test]
        fn simple() {
            let mut var = Var {name: None, format: None, data: vec![]};
            var.seq(1., 2., 2);
            assert_eq!(vec![1., 2.], var.data);
        }

        #[test]
        fn triple() {
            let mut var = Var {name: None, format: None, data: vec![]};
            var.seq(2000000000., 3000000000., 3);
            assert_eq!(vec![2000000000., 2500000000., 3000000000.], var.data);
        }

        #[test]
        fn reversed() {
            let mut var = Var {name: None, format: None, data: vec![]};
            var.seq(3000000000., 2000000000., 3);
            assert_eq!(vec![3000000000., 2500000000., 2000000000.], var.data);
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Constant {
    pub name: String,
    pub value: String,
}

impl Constant {
    pub fn new(name: &str, value: &str) -> Constant {
        Constant {
            name: String::from(name),
            value: String::from(value),
        }
    }
}

#[cfg(test)]
mod test_constant {
    use super::*;

    #[test]
    fn test_new() {
        let expected = Constant {
            name: String::from("A_NAME"),
            value: String::from("A_VALUE"),
        };
        let result = Constant::new("A_NAME", "A_VALUE");
        assert_eq!(result, expected);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    pub version: Option<String>,
    pub name: Option<String>,
    pub comments: Vec<String>,
    pub devices: Vec<Device>,
    pub independent_variable: Var,
    pub constants: Vec<Constant>,
}

impl Default for Header {
    fn default() -> Self {
        Header {
            version: Some(String::from("A.01.00")),
            name: Some(String::from("Name")),
            comments: vec![],
            devices: vec![],
            independent_variable: Var::blank(),
            constants: vec![],
        }
    }
}

impl Header {
    pub fn new(version: &str, name: &str) -> Header {
        Header {
            version: Some(String::from(version)),
            name: Some(String::from(name)),
            comments: vec![],
            devices: vec![],
            independent_variable: Var::blank(),
            constants: vec![],
        }
    }

    fn blank() -> Header {
        Header {
            version: None,
            name: None,
            comments: vec![],
            devices: vec![],
            independent_variable: Var::blank(),
            constants: vec![],
        }
    }

    pub fn add_device(&mut self, device_name: &str, value: &str) {
        self.create_device(device_name);
        match self.index_device(device_name) {
            Some(i) => self.devices[i].entries.push(String::from(value)),
            None => (), // Never occurs
        }
    }

    /// If the device already exists, nothing happens
    pub fn create_device(&mut self, device_name: &str) {
        if self.get_device_by_name(device_name) == None {
            self.devices.push(Device::new(device_name));
        }
    }

    pub fn get_device_by_name(&self, device_name: &str) -> Option<&Device> {
        self.devices.iter().find(|&x| x.name == device_name)
    }

    pub fn index_device(&self, device_name: &str) -> Option<usize> {
        self.devices.iter().position(|x| x.name == device_name)
    }
}

#[cfg(test)]
mod test_header {
    use super::*;

    #[test]
    fn test_default() {
        let expected = Header {
            version: Some(String::from("A.01.00")),
            name: Some(String::from("Name")),
            comments: vec![],
            devices: vec![],
            independent_variable: Var {name: None, format: None, data: vec![]},
            constants: vec![],
        };
        let result = Header::default();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_new() {
        let expected = Header {
            version: Some(String::from("A.01.01")),
            name: Some(String::from("A_NAME")),
            comments: vec![],
            devices: vec![],
            independent_variable: Var {name: None, format: None, data: vec![]},
            constants: vec![],
        };
        let result = Header::new("A.01.01", "A_NAME");
        assert_eq!(result, expected);
    }

    #[cfg(test)]
    mod test_devices {
        use super::*;

        #[cfg(test)]
        mod test_add_device {
            use super::*;

            #[test]
            fn empty() {
                let expected = vec![Device{name: String::from("NA"), entries: vec![String::from("VERSION HP8510B.05.00")]}];
                let mut header = Header::new("A.01.01", "A_NAME");
                header.add_device("NA", "VERSION HP8510B.05.00");
                assert_eq!(header.devices, expected);
            }

            #[test]
            fn double_add() {
                let expected = vec![Device{name: String::from("NA"), entries: vec![String::from("VERSION HP8510B.05.00"), String::from("REGISTER 1")]}];
                let mut header = Header::new("A.01.01", "A_NAME");
                header.add_device("NA", "VERSION HP8510B.05.00");
                header.add_device("NA", "REGISTER 1");
                assert_eq!(header.devices, expected);
            }

            #[test]
            fn add_two_devices() {
                let expected = vec![
                    Device{name: String::from("NA"), entries: vec![String::from("VERSION HP8510B.05.00")]},
                    Device{name: String::from("WVI"), entries: vec![String::from("REGISTER 1")]},
                ];
                let mut header = Header::new("A.01.01", "A_NAME");
                header.add_device("NA", "VERSION HP8510B.05.00");
                header.add_device("WVI", "REGISTER 1");
                assert_eq!(header.devices, expected);
            }
        }

        #[cfg(test)]
        mod test_create_device {
            use super::*;

            #[test]
            fn empty() {
                let expected = vec![Device{name: String::from("A Name"), entries: vec![]}];
                let mut header = Header::new("A.01.01", "A_NAME");
                header.create_device("A Name");
                assert_eq!(header.devices, expected);
            }

            #[test]
            fn appends_device() {
                let expected = vec![Device{name: String::from("Different Name"), entries: vec![]}, Device{name: String::from("A Name"), entries: vec![]}];
                let mut header = Header::new("A.01.01", "A_NAME");
                header.create_device("Different Name");
                header.create_device("A Name");
                assert_eq!(header.devices, expected);
            }

            #[test]
            fn existing_device() {
                let expected = vec![Device{name: String::from("A Name"), entries: vec![]}];
                let mut header = Header::new("A.01.01", "A_NAME");
                header.create_device("A Name");
                header.create_device("A Name");
                assert_eq!(header.devices, expected);
            }
        }

        #[cfg(test)]
        mod test_index_device {
            use super::*;

            #[test]
            fn empty() {
                let header = Header::new("A.01.01", "A_NAME");
                assert_eq!(header.index_device(""), None);
            }

            #[test]
            fn no_device_found() {
                let mut header = Header::new("A.01.01", "A_NAME");
                header.create_device("A Name");
                assert_eq!(header.index_device(""), None);
            }

            #[test]
            fn device_found() {
                let mut header = Header::new("A.01.01", "A_NAME");
                header.create_device("A Name");
                assert_eq!(header.index_device("A Name"), Some(0));
            }
        }

        #[cfg(test)]
        mod test_get_device_by_name {
            use super::*;

            #[test]
            fn empty() {
                let header = Header::new("A.01.01", "A_NAME");
                assert_eq!(header.get_device_by_name(""), None);
            }

            #[test]
            fn no_device_found() {
                let mut header = Header::new("A.01.01", "A_NAME");
                header.create_device("A Name");
                assert_eq!(header.get_device_by_name(""), None);
            }

            #[test]
            fn device_found() {
                let mut header = Header::new("A.01.01", "A_NAME");
                header.create_device("A Name");
                assert_eq!(header.get_device_by_name("A Name"), Some(&Device{name: String::from("A Name"), entries: vec![]}));
            }
        }
    }

}

#[derive(Debug, PartialEq, Clone)]
pub struct DataArray {
    pub name: Option<String>,
    pub format: Option<String>,
    pub real: Vec<f64>,
    pub imag: Vec<f64>,
}

impl DataArray {
    #[cfg(test)]
    fn blank() -> DataArray {
        DataArray {
            name: None,
            format: None,
            real: vec![],
            imag: vec![],
        }
    }

    pub fn new(name: &str, format: &str) -> DataArray {
        DataArray {
            name: Some(String::from(name)),
            format: Some(String::from(format)),
            real: vec![],
            imag: vec![],
        }
    }

    pub fn add_sample(&mut self, real: f64, imag: f64) {
        self.real.push(real);
        self.imag.push(imag);
    }
}

#[cfg(test)]
mod test_data_array {
    use super::*;

    #[test]
    fn test_blank() {
        let expected = DataArray {name: None, format: None, real: vec![], imag: vec![]};
        let result = DataArray::blank();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_new() {
        let expected = DataArray {name: Some(String::from("S")), format: Some(String::from("RI")), real: vec![], imag: vec![]};
        let result = DataArray::new("S", "RI");
        assert_eq!(result, expected);
    }

    #[cfg(test)]
    mod test_add_sample {
        use super::*;

        #[test]
        fn empty() {
            let mut result = DataArray {name: None, format: None, real: vec![], imag: vec![]};
            result.add_sample(1., 2.);
            assert_eq!(result.real, vec![1.]);
            assert_eq!(result.imag, vec![2.]);
        }

        #[test]
        fn double() {
            let mut result = DataArray {name: None, format: None, real: vec![], imag: vec![]};
            result.add_sample(1., 2.);
            result.add_sample(-1., -2.);
            assert_eq!(result.real, vec![1., -1.]);
            assert_eq!(result.imag, vec![2., -2.]);
        }

        #[test]
        fn existing() {
            let mut result = DataArray {name: None, format: None, real: vec![1.], imag: vec![2.]};
            result.add_sample(3., 4.);
            assert_eq!(result.real, vec![1., 3.]);
            assert_eq!(result.imag, vec![2., 4.]);
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Record {
    pub header: Header,
    pub data: Vec<DataArray>,
}

impl Default for Record {
    fn default() -> Self {
        Record {
            header: Header::default(),
            data: vec![],
        }
    }
}

#[derive(Error, Debug)]
pub enum ValidRecordError {
    #[error("Version is not defined")]
    NoVersion,
    #[error("Name is not defined")]
    NoName,
    #[error("Indepent variable is not defined")]
    NoIndependentVariable,
    #[error("Data name and format is not defined")]
    NoData,
    #[error("Independent variable and data array {2} are different lengths ({0} != {1})")]
    VarAndDataDifferentLengths(usize, usize, usize),
    #[error("Data array {2} has different length real and imaginary components ({0} != {1})")]
    RealImagDoNotMatch(usize, usize, usize),
}
type ValidRecordResult<T> = std::result::Result<T, ValidRecordError>;

#[cfg(test)]
mod test_valid_record_error {
    use super::*;

    mod test_display {
        use super::*;

        #[test]
        fn no_version() {
            let error = ValidRecordError::NoVersion;
            assert_eq!(format!("{}", error), "Version is not defined");
        }

        #[test]
        fn no_name() {
            let error = ValidRecordError::NoName;
            assert_eq!(format!("{}", error), "Name is not defined");
        }

        #[test]
        fn no_independent_variable() {
            let error = ValidRecordError::NoIndependentVariable;
            assert_eq!(format!("{}", error), "Indepent variable is not defined");
        }

        #[test]
        fn no_data() {
            let error = ValidRecordError::NoData;
            assert_eq!(format!("{}", error), "Data name and format is not defined");
        }

        #[test]
        fn var_and_data() {
            let error = ValidRecordError::VarAndDataDifferentLengths(1, 2, 3);
            assert_eq!(format!("{}", error), "Independent variable and data array 3 are different lengths (1 != 2)");
        }

        #[test]
        fn data_real_imag_do_not_match() {
            let error = ValidRecordError::RealImagDoNotMatch(1, 2, 0);
            assert_eq!(format!("{}", error), "Data array 0 has different length real and imaginary components (1 != 2)");
        }
    }
}

#[derive(Error, Debug)]
pub enum WriteError {
    #[error("Version is not defined")]
    NoVersion,
    #[error("Name is not defined")]
    NoName,
    #[error("Data array {0} has no name")]
    NoDataName(usize),
    #[error("Data array {0} has no format")]
    NoDataFormat(usize),
    #[error("Var has no name")]
    NoVarName,
    #[error("Data array {2} has different length real and imaginary components ({0} != {1})")]
    RealImagDoNotMatch(usize, usize, usize),
    #[error("Cannot write record to `{0}`: {1}")]
    CannotWrite(PathBuf, std::io::Error),
    #[error("Writing error occured: {0}")]
    WrittingError(std::io::Error),
}
type WriteResult<T> = std::result::Result<T, WriteError>;


#[cfg(test)]
mod test_write_result {
    use super::*;

    mod test_display {
        use super::*;

        #[test]
        fn no_version() {
            let error = WriteError::NoVersion;
            assert_eq!(format!("{}", error), "Version is not defined");
        }

        #[test]
        fn no_name() {
            let error = WriteError::NoName;
            assert_eq!(format!("{}", error), "Name is not defined");
        }

        #[test]
        fn no_data_name() {
            let error = WriteError::NoDataName(1);
            assert_eq!(format!("{}", error), "Data array 1 has no name");
        }

        #[test]
        fn no_data_name_second() {
            let error = WriteError::NoDataName(2);
            assert_eq!(format!("{}", error), "Data array 2 has no name");
        }

        #[test]
        fn no_data_format() {
            let error = WriteError::NoDataFormat(1);
            assert_eq!(format!("{}", error), "Data array 1 has no format");
        }

        #[test]
        fn no_data_format_second() {
            let error = WriteError::NoDataFormat(2);
            assert_eq!(format!("{}", error), "Data array 2 has no format");
        }

        #[test]
        fn no_var_name() {
            let error = WriteError::NoVarName;
            assert_eq!(format!("{}", error), "Var has no name");
        }

        #[test]
        fn real_image_do_not_match() {
            let error = WriteError::RealImagDoNotMatch(0, 1, 2);
            assert_eq!(format!("{}", error), "Data array 2 has different length real and imaginary components (0 != 1)");
        }

        #[test]
        fn cannot_write() {
            let error = WriteError::CannotWrite(Path::new("/temp").to_path_buf(), std::io::ErrorKind::NotFound.into());
            assert_eq!(format!("{}", error), "Cannot write record to `/temp`: entity not found");
        }

        #[test]
        fn writting_error() {
            let error = WriteError::WrittingError(std::io::ErrorKind::NotFound.into());
            assert_eq!(format!("{}", error), "Writing error occured: entity not found");
        }
    }
}

impl Record {
    pub fn new(version: &str, name: &str) -> Record {
        Record {
            header: Header::new(version, name),
            data: vec![],
        }
    }

    pub fn read<P: AsRef<Path>>(path: &P)  -> Result<Record> {
        let contents = std::fs::read_to_string(path).map_err(|_| ReaderError::CannotOpen(path.as_ref().to_path_buf()))?;
        Record::read_str(&contents)
    }

    fn read_str(contents: &str) -> Result<Record> {
        let mut state = RecordReaderState::new();
        for (i, line) in contents.lines().enumerate() {
            // Filter out new lines
            if line.trim().len() > 0 {
                let keyword = Keywords::try_from(line).map_err(|e| ReaderError::LineError(i, e))?;
                state = state.process_keyword(keyword)?;
            }
        }
        Ok(state.record.validate_record()?)
    }

    pub fn write<P: AsRef<Path>>(&self, path: &P)  -> Result<()> {
        let mut buffer = std::io::BufWriter::new(std::fs::File::create(path).map_err(|e| WriteError::CannotWrite(path.as_ref().to_path_buf(), e))?);
        self.write_to_sink(&mut buffer)
    }

    pub fn write_to_sink<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        let keywords = self.get_keywords()?;
        
        for keyword in keywords.iter() {
            writeln!(writer, "{}", keyword).map_err(|e| WriteError::WrittingError(e))?;
        }

        Ok(())
    }

    fn get_data_keywords(&self) -> WriteResult<Vec<Keywords>> {
        let mut keywords: Vec<Keywords> = vec![];

        // Add each array
        for (i, array) in self.data.iter().enumerate() {
            keywords.push(Keywords::Begin);

            // Check lengths
            if array.real.len() != array.imag.len() {
                return Err(WriteError::RealImagDoNotMatch(array.real.len(), array.imag.len(), i));
            }

            for (&real, &imag) in array.real.iter().zip(array.imag.iter()) {
                keywords.push(Keywords::DataPair{real: real, imag: imag});
            }
            keywords.push(Keywords::End);
        }

        Ok(keywords)
    }

    fn get_data_defines_keywords(&self) -> WriteResult<Vec<Keywords>> {
        let mut keywords: Vec<Keywords> = vec![];

        for (i, array) in self.data.iter().enumerate() {
            match (&array.name, &array.format) {
                (Some(ref n), Some(ref f)) => keywords.push(Keywords::Data{name: n.clone(), format: f.clone()}),
                (None, _) => return Err(WriteError::NoDataName(i)),
                (_, None) => return Err(WriteError::NoDataFormat(i)),
            }
        }
        Ok(keywords)
    }

    fn get_version_keywords(&self) -> WriteResult<Vec<Keywords>> {
        match self.header.version {
            Some(ref v) => Ok(vec![Keywords::CITIFile{version: v.clone()}]),
            None => Err(WriteError::NoVersion),
        }
    }

    fn get_name_keywords(&self) -> WriteResult<Vec<Keywords>> {
        match self.header.name {
            Some(ref v) => Ok(vec![Keywords::Name(v.clone())]),
            None => Err(WriteError::NoName),
        }
    }

    fn get_comments_keywords(&self) -> WriteResult<Vec<Keywords>> {
        Ok(self.header.comments.iter().map(|s| Keywords::Comment(s.clone())).collect())
    }

    fn get_devices_keywords(&self) -> WriteResult<Vec<Keywords>> {
        let mut keywords: Vec<Keywords> = vec![];

        for device in self.header.devices.iter() {
            for entry in device.entries.iter() {
                keywords.push(Keywords::Device{name: device.name.clone(), value: entry.clone()});
            }
        }

        Ok(keywords)
    }

    fn get_independent_variable_keywords(&self) -> WriteResult<Vec<Keywords>> {
        match self.header.independent_variable.name {
            Some(ref n) => Ok(vec![Keywords::Var{
                name: n.clone(),
                format: self.header.independent_variable.format.clone(),
                length: self.header.independent_variable.data.len()
            }]),
            None => Err(WriteError::NoVarName)
        }
    }

    fn get_var_keywords(&self) -> WriteResult<Vec<Keywords>> {
        let mut keywords: Vec<Keywords> = vec![];

        // Do not set if length == 0
        if self.header.independent_variable.data.len() > 0 {
            keywords.push(Keywords::VarListBegin);
            for &v in self.header.independent_variable.data.iter() {
                keywords.push(Keywords::VarListItem(v));
            }
            keywords.push(Keywords::VarListEnd);
        }

        Ok(keywords)
    }

    fn get_constants_keywords(&self) -> WriteResult<Vec<Keywords>> {
        Ok(self.header.constants.iter().map(|c| Keywords::Constant{name: c.name.clone(), value: c.value.clone()}).collect())
    }

    fn get_keywords(&self) -> WriteResult<Vec<Keywords>> {
        let mut keywords: Vec<Keywords> = vec![];

        keywords.append(&mut self.get_version_keywords()?);
        keywords.append(&mut self.get_name_keywords()?);
        keywords.append(&mut self.get_independent_variable_keywords()?);
        keywords.append(&mut self.get_var_keywords()?);
        keywords.append(&mut self.get_constants_keywords()?);
        keywords.append(&mut self.get_comments_keywords()?);
        keywords.append(&mut self.get_devices_keywords()?);
        keywords.append(&mut self.get_data_defines_keywords()?);
        keywords.append(&mut self.get_data_keywords()?);

        Ok(keywords)
    }

    #[cfg(test)]
    fn blank() -> Record {
        Record {
            header: Header::blank(),
            data: vec![],
        }
    }

    pub fn validate_record(self) -> ValidRecordResult<Self> {
        self.has_name()?
            .has_version()?
            .has_var()?
            .has_data()?
            .var_and_data_same_length()?
            .data_real_image_same_length()
    }

    fn has_version(self) -> ValidRecordResult<Self> {
        match self.header.version {
            Some(_) => Ok(self),
            None => Err(ValidRecordError::NoVersion),
        }
    }

    fn has_name(self) -> ValidRecordResult<Self> {
        match self.header.name {
            Some(_) => Ok(self),
            None => Err(ValidRecordError::NoName),
        }
    }

    fn has_var(self) -> ValidRecordResult<Self> {
        match self.header.independent_variable.name {
            Some(_) => Ok(self),
            None => Err(ValidRecordError::NoIndependentVariable),
        }
    }

    fn has_data(self) -> ValidRecordResult<Self> {
        match self.data.len() {
            0 => Err(ValidRecordError::NoData),
            _ => Ok(self),
        }
    }

    /// Zero length var with variable length data allowed
    fn var_and_data_same_length(self) -> ValidRecordResult<Self> {
        let mut n = self.header.independent_variable.data.len();

        for (i, data_array) in self.data.iter().enumerate() {
            let k = data_array.real.len();
            if n == 0 {
                n = k
            } else {
                if n != k {
                    return Err(ValidRecordError::VarAndDataDifferentLengths(n, k, i))
                }
            }
        }
        Ok(self)
    }

    fn data_real_image_same_length(self) -> ValidRecordResult<Self> {
        for (i, data_array) in self.data.iter().enumerate() {
            let real_size = data_array.real.len();
            let imag_size = data_array.imag.len();

            if real_size != imag_size {
                return Err(ValidRecordError::RealImagDoNotMatch(real_size, imag_size, i));
            }
        }
        Ok(self)
    }
}

#[cfg(test)]
mod test_record {
    use super::*;

    #[test]
    fn write_gives_error_on_bad_file() {
        let record = Record::default();
        match record.write(&Path::new("")) {
            Err(Error::WriteError(WriteError::CannotWrite(path, std::io::Error{..}))) => assert_eq!(path, Path::new("").to_path_buf()),
            e => panic!("{:?}", e),
        }
    }

    mod test_write {
        use super::*;

        #[test]
        fn get_keywords() {
            let mut record = Record::default();
            record.header.constants.push(Constant{name: String::from("Const Name"), value: String::from("Value")});
            record.header.independent_variable = Var{name: Some(String::from("Var Name")), format: Some(String::from("Format")), data: vec![1.]};
            record.header.devices.push(Device{name: String::from("Name A"), entries: vec![String::from("entry 1"), String::from("entry 2")]});
            record.header.comments.push(String::from("A Comment"));
            record.header.name = Some(String::from("Name"));
            record.header.version = Some(String::from("A.01.00"));
            record.data.push(DataArray{name: Some(String::from("Data Name A")), format: Some(String::from("Format A")), real: vec![1.], imag: vec![2.]});
            record.data.push(DataArray{name: Some(String::from("Data Name B")), format: Some(String::from("Format B")), real: vec![3., 4.], imag: vec![5., 6.]});

            match record.get_keywords() {
                Ok(v) => assert_eq!(v, vec![
                    Keywords::CITIFile{version: String::from("A.01.00")},
                    Keywords::Name(String::from("Name")),
                    Keywords::Var{name: String::from("Var Name"), format: Some(String::from("Format")), length: 1},
                    Keywords::VarListBegin,
                    Keywords::VarListItem(1.),
                    Keywords::VarListEnd,
                    Keywords::Constant{name: String::from("Const Name"), value: String::from("Value")},
                    Keywords::Comment(String::from("A Comment")),
                    Keywords::Device{name: String::from("Name A"), value: String::from("entry 1")},
                    Keywords::Device{name: String::from("Name A"), value: String::from("entry 2")},
                    Keywords::Data{name: String::from("Data Name A"), format: String::from("Format A")},
                    Keywords::Data{name: String::from("Data Name B"), format: String::from("Format B")},
                    Keywords::Begin,
                    Keywords::DataPair{real: 1., imag: 2.},
                    Keywords::End,
                    Keywords::Begin,
                    Keywords::DataPair{real: 3., imag: 5.},
                    Keywords::DataPair{real: 4., imag: 6.},
                    Keywords::End,
                ]),
                e => panic!("{:?}", e),
            }
        }

        mod test_get_var_keywords {
            use super::*;

            #[test]
            fn empty() {
                let record = Record::default();
                match record.get_var_keywords() {
                    Ok(v) => assert_eq!(v, vec![]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn one() {
                let mut record = Record::default();
                record.header.independent_variable.data.push(1.);
                match record.get_var_keywords() {
                    Ok(v) => assert_eq!(v, vec![
                        Keywords::VarListBegin,
                        Keywords::VarListItem(1.),
                        Keywords::VarListEnd
                    ]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn multiple() {
                let mut record = Record::default();
                record.header.independent_variable.data.push(1.);
                record.header.independent_variable.data.push(2.);
                record.header.independent_variable.data.push(3.);
                match record.get_var_keywords() {
                    Ok(v) => assert_eq!(v, vec![
                        Keywords::VarListBegin,
                        Keywords::VarListItem(1.),
                        Keywords::VarListItem(2.),
                        Keywords::VarListItem(3.),
                        Keywords::VarListEnd
                    ]),
                    e => panic!("{:?}", e),
                }
            }
        }

        mod test_get_constants_keywords {
            use super::*;

            #[test]
            fn empty() {
                let record = Record::default();
                match record.get_constants_keywords() {
                    Ok(v) => assert_eq!(v, vec![]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn one() {
                let mut record = Record::default();
                record.header.constants.push(Constant{name: String::from("Name"), value: String::from("Value")});
                match record.get_constants_keywords() {
                    Ok(v) => assert_eq!(v, vec![Keywords::Constant{name: String::from("Name"), value: String::from("Value")}]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn two() {
                let mut record = Record::default();
                record.header.constants.push(Constant{name: String::from("Name A"), value: String::from("Value A")});
                record.header.constants.push(Constant{name: String::from("Name B"), value: String::from("Value B")});
                match record.get_constants_keywords(){
                    Ok(v) => assert_eq!(v, vec![
                        Keywords::Constant{name: String::from("Name A"), value: String::from("Value A")},
                        Keywords::Constant{name: String::from("Name B"), value: String::from("Value B")}
                    ]),
                    e => panic!("{:?}", e),
                }
            }
        }

        mod test_get_independent_variable_keywords {
            use super::*;

            #[test]
            fn no_name() {
                let record = Record::default();
                match record.get_independent_variable_keywords() {
                    Err(WriteError::NoVarName) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn no_format() {
                let mut record = Record::default();
                record.header.independent_variable.name = Some(String::from("Name"));
                match record.get_independent_variable_keywords() {
                    Ok(v) => assert_eq!(v, vec![Keywords::Var{
                        name: String::from("Name"),
                        format: None,
                        length: 0
                    }]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn format() {
                let mut record = Record::default();
                record.header.independent_variable.name = Some(String::from("Name"));
                record.header.independent_variable.format = Some(String::from("Format"));
                match record.get_independent_variable_keywords() {
                    Ok(v) => assert_eq!(v, vec![Keywords::Var{
                        name: String::from("Name"),
                        format: Some(String::from("Format")),
                        length: 0
                    }]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn length() {
                let mut record = Record::default();
                record.header.independent_variable.name = Some(String::from("Name"));
                record.header.independent_variable.format = Some(String::from("Format"));
                record.header.independent_variable.data = vec![0.; 10];
                match record.get_independent_variable_keywords() {
                    Ok(v) => assert_eq!(v, vec![Keywords::Var{
                        name: String::from("Name"),
                        format: Some(String::from("Format")),
                        length: 10
                    }]),
                    e => panic!("{:?}", e),
                }
            }
        }

        mod test_get_devices_keywords {
            use super::*;

            #[test]
            fn empty() {
                let record = Record::default();
                match record.get_devices_keywords() {
                    Ok(v) => assert_eq!(v, vec![]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn one_device_no_entry() {
                let mut record = Record::default();
                record.header.devices.push(Device{name: String::from(""), entries: vec![]});
                match record.get_devices_keywords() {
                    Ok(v) => assert_eq!(v, vec![]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn one_device() {
                let mut record = Record::default();
                record.header.devices.push(Device{name: String::from("Name"), entries: vec![String::from("entry")]});
                match record.get_devices_keywords() {
                    Ok(v) => assert_eq!(v, vec![
                        Keywords::Device{name: String::from("Name"), value: String::from("entry")}
                    ]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn one_device_multiple_entries() {
                let mut record = Record::default();
                record.header.devices.push(Device{name: String::from("Name"), entries: vec![String::from("entry 1"), String::from("entry 2")]});
                match record.get_devices_keywords() {
                    Ok(v) => assert_eq!(v, vec![
                        Keywords::Device{name: String::from("Name"), value: String::from("entry 1")},
                        Keywords::Device{name: String::from("Name"), value: String::from("entry 2")}
                    ]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn multiple_devices() {
                let mut record = Record::default();
                record.header.devices.push(Device{name: String::from("Name A"), entries: vec![String::from("entry 1")]});
                record.header.devices.push(Device{name: String::from("Name B"), entries: vec![String::from("entry 2")]});
                match record.get_devices_keywords() {
                    Ok(v) => assert_eq!(v, vec![
                        Keywords::Device{name: String::from("Name A"), value: String::from("entry 1")},
                        Keywords::Device{name: String::from("Name B"), value: String::from("entry 2")}
                    ]),
                    e => panic!("{:?}", e),
                }
            }
        }

        mod test_get_comments_keywords {
            use super::*;

            #[test]
            fn empty() {
                let record = Record::default();
                match record.get_comments_keywords() {
                    Ok(v) => assert_eq!(v, vec![]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn one() {
                let mut record = Record::default();
                record.header.comments.push(String::from("A Comment"));
                match record.get_comments_keywords() {
                    Ok(v) => assert_eq!(v, vec![Keywords::Comment(String::from("A Comment"))]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn multiple() {
                let mut record = Record::default();
                record.header.comments.push(String::from("A Comment"));
                record.header.comments.push(String::from("B Comment"));
                match record.get_comments_keywords() {
                    Ok(v) => assert_eq!(v, vec![
                        Keywords::Comment(String::from("A Comment")),
                        Keywords::Comment(String::from("B Comment"))
                    ]),
                    e => panic!("{:?}", e),
                }
            }
        }

        mod test_get_name_keywords {
            use super::*;

            #[test]
            fn none() {
                let mut record = Record::default();
                record.header.name = None;
                match record.get_name_keywords() {
                    Err(WriteError::NoName) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn some() {
                let mut record = Record::default();
                record.header.name = Some(String::from("A.01.00"));
                match record.get_name_keywords() {
                    Ok(v) => assert_eq!(v, vec![Keywords::Name(String::from("A.01.00"))]),
                    e => panic!("{:?}", e),
                }
            }
        }

        mod test_get_version_keywords {
            use super::*;

            #[test]
            fn none() {
                let mut record = Record::default();
                record.header.version = None;
                match record.get_version_keywords() {
                    Err(WriteError::NoVersion) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn some() {
                let mut record = Record::default();
                record.header.version = Some(String::from("A.01.00"));
                match record.get_version_keywords() {
                    Ok(v) => assert_eq!(v, vec![Keywords::CITIFile{version: String::from("A.01.00")}]),
                    e => panic!("{:?}", e),
                }
            }
        }

        mod test_get_data_keywords {
            use super::*;

            #[test]
            fn many_values() {
                let mut record = Record::default();
                record.data.push(DataArray{name: None, format: None, real: vec![1., 2., -3.], imag: vec![4., 1e-6, 0.]});
                match record.get_data_keywords() {
                    Ok(v) => assert_eq!(v, vec![
                        Keywords::Begin,
                        Keywords::DataPair{real: 1., imag: 4.},
                        Keywords::DataPair{real: 2., imag: 1e-6},
                        Keywords::DataPair{real: -3., imag: 0.},
                        Keywords::End
                    ]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn one_array_gives_correct_result() {
                let mut record = Record::default();
                record.data.push(DataArray{name: None, format: None, real: vec![1.], imag: vec![2.]});
                match record.get_data_keywords() {
                    Ok(v) => assert_eq!(v, vec![Keywords::Begin, Keywords::DataPair{real: 1., imag: 2.}, Keywords::End]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn multiple_array_gives_correct_result() {
                let mut record = Record::default();
                record.data.push(DataArray{name: None, format: None, real: vec![1.], imag: vec![2.]});
                record.data.push(DataArray{name: None, format: None, real: vec![3.], imag: vec![4.]});
                match record.get_data_keywords() {
                    Ok(v) => assert_eq!(v, vec![
                            Keywords::Begin, Keywords::DataPair{real: 1., imag: 2.}, Keywords::End,
                            Keywords::Begin, Keywords::DataPair{real: 3., imag: 4.}, Keywords::End
                        ]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn no_arrays_gives_empty() {
                let record = Record::default();
                match record.get_data_keywords() {
                    Ok(v) => assert_eq!(v, vec![]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn bad_real_imag_gives_error() {
                let mut record = Record::default();
                record.data.push(DataArray{name: None, format: None, real: vec![1., 2.], imag: vec![1.]});
                match record.get_data_keywords() {
                    Err(WriteError::RealImagDoNotMatch(2, 1, 0)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn bad_real_imag_gives_error_with_correct_array_count() {
                let mut record = Record::default();
                record.data.push(DataArray{name: None, format: None, real: vec![1., 2.], imag: vec![1., 2.]});
                record.data.push(DataArray{name: None, format: None, real: vec![1., 2., 3.], imag: vec![1., 2.]});
                match record.get_data_keywords() {
                    Err(WriteError::RealImagDoNotMatch(3, 2, 1)) => (),
                    e => panic!("{:?}", e),
                }
            }
        }

        mod test_get_data_defines_keywords {
            use super::*;

            #[test]
            fn one_entry() {
                let mut record = Record::default();
                record.data.push(DataArray{name: Some(String::from("Name")), format: Some(String::from("Format")), real: vec![], imag: vec![]});
                match record.get_data_defines_keywords() {
                    Ok(v) => assert_eq!(v, vec![Keywords::Data{name: String::from("Name"), format: String::from("Format")}]),
                    _ => panic!("One entry fails"),
                }
            }

            #[test]
            fn multiple() {
                let mut record = Record::default();
                record.data.push(DataArray{name: Some(String::from("Name A")), format: Some(String::from("Format A")), real: vec![], imag: vec![]});
                record.data.push(DataArray{name: Some(String::from("Name B")), format: Some(String::from("Format B")), real: vec![], imag: vec![]});
                match record.get_data_defines_keywords() {
                    Ok(v) => {
                        assert_eq!(v, vec![
                            Keywords::Data{name: String::from("Name A"), format: String::from("Format A")},
                            Keywords::Data{name: String::from("Name B"), format: String::from("Format B")}
                        ]);
                    },
                    _ => panic!("Multiple entries fails"),
                }
            }

            #[test]
            fn no_name() {
                let mut record = Record::default();
                record.data.push(DataArray{name: None, format: Some(String::from("")), real: vec![], imag: vec![]});
                match record.get_data_defines_keywords() {
                    Err(WriteError::NoDataName(0)) => (),
                    _ => panic!("No name gives incorrect error"),
                }
            }

            #[test]
            fn no_format() {
                let mut record = Record::default();
                record.data.push(DataArray{name: Some(String::from("")), format: None, real: vec![], imag: vec![]});
                match record.get_data_defines_keywords() {
                    Err(WriteError::NoDataFormat(0)) => (),
                    _ => panic!("No format gives incorrect error"),
                }
            }

            #[test]
            fn no_name_no_format() {
                let mut record = Record::default();
                record.data.push(DataArray{name: None, format: None, real: vec![], imag: vec![]});
                match record.get_data_defines_keywords() {
                    Err(WriteError::NoDataName(0)) => (),
                    _ => panic!("No name no format does not give correct error"),
                }
            }
        }
    }

    #[cfg(test)]
    mod test_read {
        use super::*;
        use approx::*;

        #[test]
        fn cannot_read_empty_record() {
            match Record::read_str("") {
                Err(Error::ValidRecordError(ValidRecordError::NoName)) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn succeed_on_multiple_new_lines() {
            let contents = "CITIFILE A.01.00\nNAME MEMORY\n\n\n\n\n\n\n\n\nVAR FREQ MAG 3\nDATA S RI\nBEGIN\n-3.54545E-2,-1.38601E-3\n0.23491E-3,-1.39883E-3\n2.00382E-3,-1.40022E-3\nEND\n";
            match Record::read_str(contents) {
                Ok(_) => (),
                Err(_) => panic!("Cannot parse when there are multiple blank lines"),
            }
        }

        #[test]
        fn succeed_on_whitespace_new_lines() {
            let contents = "CITIFILE A.01.00\nNAME MEMORY\n      \n\n\n\n\n\n\n\nVAR FREQ MAG 3\nDATA S RI\nBEGIN\n-3.54545E-2,-1.38601E-3\n0.23491E-3,-1.39883E-3\n2.00382E-3,-1.40022E-3\nEND\n";
            match Record::read_str(contents) {
                Ok(_) => (),
                Err(_) => panic!("Cannot parse when there are multiple blank lines"),
            }
        }

        #[cfg(test)]
        mod test_read_minimal_record {
            use super::*;

            fn setup() -> Result<Record> {
                let contents = "CITIFILE A.01.00\nNAME MEMORY\nVAR FREQ MAG 3\nDATA S RI\nBEGIN\n-3.54545E-2,-1.38601E-3\n0.23491E-3,-1.39883E-3\n2.00382E-3,-1.40022E-3\nEND\n";
                Record::read_str(contents)
            }

            #[test]
            fn name() {
                match setup() {
                    Ok(record) => assert_eq!(record.header.name, Some(String::from("MEMORY"))),
                    Err(_) => panic!("File could not be read"),
                }
            }

            #[test]
            fn version() {
                match setup() {
                    Ok(record) => assert_eq!(record.header.version, Some(String::from("A.01.00"))),
                    Err(_) => panic!("File could not be read"),
                }
            }

            #[test]
            fn comments() {
                match setup() {
                    Ok(record) => assert_eq!(record.header.comments.len(), 0),
                    Err(_) => panic!("File could not be read"),
                }
            }

            #[test]
            fn constants() {
                match setup() {
                    Ok(record) => assert_eq!(record.header.constants.len(), 0),
                    Err(_) => panic!("File could not be read"),
                }
            }

            #[test]
            fn devices() {
                match setup() {
                    Ok(record) => assert_eq!(record.header.devices.len(), 0),
                    Err(_) => panic!("File could not be read"),
                }
            }

            #[test]
            fn independent_variable() {
                match setup() {
                    Ok(record) => {
                        assert_eq!(record.header.independent_variable.name, Some(String::from("FREQ")));
                        assert_eq!(record.header.independent_variable.format, Some(String::from("MAG")));
                        assert_eq!(record.header.independent_variable.data.len(), 0);
                    },
                    Err(_) => panic!("File could not be read"),
                }
            }

            #[test]
            fn data() {
                match setup() {
                    Ok(record) => {
                        assert_eq!(record.data.len(), 1);
                        assert_eq!(record.data[0].name, Some(String::from("S")));
                        assert_eq!(record.data[0].format, Some(String::from("RI")));
                        assert_eq!(record.data[0].real.len(), 3);
                        assert_eq!(record.data[0].imag.len(), 3);
                        assert_relative_eq!(record.data[0].real[0], -0.0354545);
                        assert_relative_eq!(record.data[0].real[1], 0.00023491);
                        assert_relative_eq!(record.data[0].real[2], 0.00200382);
                        assert_relative_eq!(record.data[0].imag[0], -0.00138601);
                        assert_relative_eq!(record.data[0].imag[1], -0.00139883);
                        assert_relative_eq!(record.data[0].imag[2], -0.00140022);
                    },
                    Err(_) => panic!("File could not be read"),
                }
            }
        }
    }

    #[cfg(test)]
    mod test_validate_record {
        use super::*;

        fn create_valid_record() -> Record {
            let mut record = Record::blank();
            record.header.name = Some(String::from("CAL_SET"));
            record.header.version = Some(String::from("A.01.00"));
            record.data.push(DataArray::new("E", "RI"));
            record.header.independent_variable.name = Some(String::from("FREQ"));
            record
        }

        #[test]
        fn test_valid_record() {
            let record = create_valid_record();
            match record.validate_record() {
                Ok(r) => assert_eq!(r, create_valid_record()),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn test_no_data() {
            let mut record = create_valid_record();
            record.data = vec![];
            match record.validate_record() {
                Err(ValidRecordError::NoData) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn test_no_version() {
            let mut record = create_valid_record();
            record.header.version = None;
            match record.validate_record() {
                Err(ValidRecordError::NoVersion) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn test_no_name() {
            let mut record = create_valid_record();
            record.header.name = None;
            match record.validate_record() {
                Err(ValidRecordError::NoName) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn test_no_var() {
            let mut record = create_valid_record();
            record.header.independent_variable.name = None;
            match record.validate_record() {
                Err(ValidRecordError::NoIndependentVariable) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn test_var_and_data_different() {
            let mut record = create_valid_record();
            record.header.independent_variable.data = vec![1.];
            match record.validate_record() {
                Err(ValidRecordError::VarAndDataDifferentLengths(1, 0, 0)) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn test_real_imag_do_not_match() {
            let mut record = create_valid_record();
            record.data[0] = DataArray{
                name: None,
                format: None,
                real: vec![1., 2., 3.],
                imag: vec![1., 2., 3., 4.],
            };
            record.header.independent_variable.data = vec![1., 2., 3.];
            match record.validate_record() {
                Err(ValidRecordError::RealImagDoNotMatch(3, 4, 0)) => (),
                e => panic!("{:?}", e),
            }
        }

        #[cfg(test)]
        mod test_has_data {
            use super::*;

            #[test]
            fn fail_on_no_version() {
                let record = Record::blank();
                match record.has_data() {
                    Err(ValidRecordError::NoData) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn pass_on_name() {
                let mut expected = Record::blank();
                expected.data.push(DataArray::new("E", "RI"));
                let mut record = Record::blank();
                record.data.push(DataArray::new("E", "RI"));
                match record.has_data() {
                    Ok(r) => assert_eq!(r, expected),
                    e => panic!("{:?}", e),
                }
            }
        }

        #[cfg(test)]
        mod test_has_var {
            use super::*;

            #[test]
            fn fail_on_no_version() {
                let record = Record::blank();
                match record.has_var() {
                    Err(ValidRecordError::NoIndependentVariable) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn pass_on_name() {
                let mut expected = Record::blank();
                expected.header.independent_variable.name = Some(String::from("FREQ"));
                let mut record = Record::blank();
                record.header.independent_variable.name = Some(String::from("FREQ"));
                match record.has_var() {
                    Ok(r) => assert_eq!(r, expected),
                    e => panic!("{:?}", e),
                }
            }
        }

        #[cfg(test)]
        mod test_has_version {
            use super::*;

            #[test]
            fn fail_on_no_version() {
                let record = Record::blank();
                match record.has_version() {
                    Err(ValidRecordError::NoVersion) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn pass_on_name() {
                let mut expected = Record::blank();
                expected.header.version = Some(String::from("A.01.00"));
                let mut record = Record::blank();
                record.header.version = Some(String::from("A.01.00"));
                match record.has_version() {
                    Ok(r) => assert_eq!(r, expected),
                    e => panic!("{:?}", e),
                }
            }
        }

        #[cfg(test)]
        mod test_has_name {
            use super::*;

            #[test]
            fn fail_on_no_name() {
                let record = Record::blank();
                match record.has_name() {
                    Err(ValidRecordError::NoName) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn pass_on_name() {
                let mut expected = Record::blank();
                expected.header.name = Some(String::from("CAL_SET"));
                let mut record = Record::blank();
                record.header.name = Some(String::from("CAL_SET"));
                match record.has_name() {
                    Ok(r) => assert_eq!(r, expected),
                    e => panic!("{:?}", e),
                }
            }
        }

        #[cfg(test)]
        mod test_var_data_different_lengths {
            use super::*;

            #[test]
            fn pass_on_blank() {
                let record = Record::blank();
                match record.var_and_data_same_length() {
                    Ok(r) => assert_eq!(r, Record::blank()),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn pass_on_equal() {
                let mut record = Record::blank();
                record.data.push(DataArray{
                    name: None,
                    format: None,
                    real: vec![1.],
                    imag: vec![1.],
                });
                record.header.independent_variable.data = vec![1.];
                match record.clone().var_and_data_same_length() {
                    Ok(r) => assert_eq!(r, record),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn pass_on_var_zero_data_some() {
                let mut record = Record::blank();
                record.data.push(DataArray{
                    name: None,
                    format: None,
                    real: vec![1., 2.],
                    imag: vec![1., 2.],
                });
                match record.clone().var_and_data_same_length() {
                    Ok(r) => assert_eq!(r, record),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn fail_on_var_one_data_some() {
                let mut record = Record::blank();
                record.data.push(DataArray{
                    name: None,
                    format: None,
                    real: vec![1., 2.],
                    imag: vec![1., 2.],
                });
                record.header.independent_variable.data = vec![1.];
                match record.var_and_data_same_length() {
                    Err(ValidRecordError::VarAndDataDifferentLengths(1, 2, 0)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn error_formatted_correctely() {
                let mut record = Record::blank();
                record.data.push(DataArray{
                    name: None,
                    format: None,
                    real: vec![1.],
                    imag: vec![1.],
                });
                record.data.push(DataArray{
                    name: None,
                    format: None,
                    real: vec![1., 2.],
                    imag: vec![1., 2.],
                });
                record.header.independent_variable.data = vec![1.];
                match record.var_and_data_same_length() {
                    Err(ValidRecordError::VarAndDataDifferentLengths(1, 2, 1)) => (),
                    e => panic!("{:?}", e),
                }
            }
        }

        #[cfg(test)]
        mod test_data_real_image_same_length {
            use super::*;

            #[test]
            fn fail_on_different_double_array() {
                let mut record = Record::blank();
                record.data.push(DataArray{
                    name: None,
                    format: None,
                    real: vec![],
                    imag: vec![],
                });
                record.data.push(DataArray{
                    name: None,
                    format: None,
                    real: vec![1., 2., 3.],
                    imag: vec![1., 2.],
                });
                match record.data_real_image_same_length() {
                    Err(ValidRecordError::RealImagDoNotMatch(3, 2, 1)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn fail_on_different() {
                let mut record = Record::blank();
                record.data.push(DataArray{
                    name: None,
                    format: None,
                    real: vec![1., 2., 3.],
                    imag: vec![1., 2.],
                });
                match record.data_real_image_same_length() {
                    Err(ValidRecordError::RealImagDoNotMatch(3, 2, 0)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn pass_on_empty() {
                let record = Record::blank();
                match record.clone().data_real_image_same_length() {
                    Ok(r) => assert_eq!(r, record),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn pass_on_equal() {
                let mut record = Record::blank();
                record.data.push(DataArray{
                    name: None,
                    format: None,
                    real: vec![1., 2.],
                    imag: vec![1., 2.],
                });
                match record.clone().data_real_image_same_length() {
                    Ok(r) => assert_eq!(r, record),
                    e => panic!("{:?}", e),
                }
            }
        }
    }

    #[test]
    fn test_default() {
        let expected = Record {
            header: Header {
                version: Some(String::from("A.01.00")),
                name: Some(String::from("Name")),
                comments: vec![],
                devices: vec![],
                independent_variable: Var {name: None, format: None, data: vec![]},
                constants: vec![],
            },
            data: vec![],
        };
        let result = Record::default();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_new() {
        let expected = Record {
            header: Header {
                version: Some(String::from("A.01.01")),
                name: Some(String::from("A_NAME")),
                comments: vec![],
                devices: vec![],
                independent_variable: Var {name: None, format: None, data: vec![]},
                constants: vec![],
            },
            data: vec![],
        };
        let result = Record::new("A.01.01", "A_NAME");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_blank() {
        let expected = Record {
            header: Header {
                version: None,
                name: None,
                comments: vec![],
                devices: vec![],
                independent_variable: Var {name: None, format: None, data: vec![]},
                constants: vec![],
            },
            data: vec![],
        };
        let result = Record::blank();
        assert_eq!(result, expected);
    }
}

#[derive(Error, Debug)]
pub enum ReaderError {
    #[error("More data arrays than defined in header")]
    DataArrayOverIndex,
    #[error("Independent variable defined twice")]
    IndependentVariableDefinedTwice,
    #[error("Single use keyword `{0}` defined twice")]
    SingleUseKeywordDefinedTwice(Keywords),
    #[error("Keyword `{0}` is out of order in the record")]
    OutOfOrderKeyword(Keywords),
    #[error("Cannot open record `{0}`")]
    CannotOpen(PathBuf),
    #[error("Error on line {0}: {1}")]
    LineError(usize, ParseError),
}
type ReaderResult<T> = std::result::Result<T, ReaderError>;

#[cfg(test)]
mod test_reader_error {
    use super::*;

    mod test_display {
        use super::*;

        #[test]
        fn data_array_over_index() {
            let error = ReaderError::DataArrayOverIndex;
            assert_eq!(format!("{}", error), "More data arrays than defined in header");
        }

        #[test]
        fn independent_variable_defined_twice() {
            let error = ReaderError::IndependentVariableDefinedTwice;
            assert_eq!(format!("{}", error), "Independent variable defined twice");
        }

        #[test]
        fn single_use_keyword_defined_twice() {
            let error = ReaderError::SingleUseKeywordDefinedTwice(Keywords::End);
            assert_eq!(format!("{}", error), "Single use keyword `END` defined twice");
        }

        #[test]
        fn out_of_order_keyword() {
            let error = ReaderError::OutOfOrderKeyword(Keywords::Begin);
            assert_eq!(format!("{}", error), "Keyword `BEGIN` is out of order in the record");
        }

        #[test]
        fn cannot_open() {
            let error = ReaderError::CannotOpen(Path::new("/temp").to_path_buf());
            assert_eq!(format!("{}", error), "Cannot open record `/temp`");
        }

        #[test]
        fn line_error() {
            let error = ReaderError::LineError(10, ParseError::BadRegex);
            assert_eq!(format!("{}", error), "Error on line 10: Regex could not be parsed");
        }
    }
}

/// States in the reader FSM
#[derive(Debug, PartialEq, Clone, Copy)]
enum RecordReaderStates {
    Header,
    Data,
    VarList,
    SeqList,
}

/// Represents state in a CITI record reader FSM
#[derive(Debug, PartialEq)]
struct RecordReaderState {
    record: Record,
    state: RecordReaderStates,
    data_array_counter: usize,
    independent_variable_already_read: bool,
}

impl RecordReaderState {
    pub fn new() -> RecordReaderState {
        RecordReaderState {
            record: Record {
                header: Header::blank(),
                data: vec![],
            },
            state: RecordReaderStates::Header,
            data_array_counter: 0,
            independent_variable_already_read: false,
        }
    }

    pub fn process_keyword(self, keyword: Keywords) -> ReaderResult<Self> {
        match self.state {
            RecordReaderStates::Header => RecordReaderState::state_header(self, keyword),
            RecordReaderStates::Data => RecordReaderState::state_data(self, keyword),
            RecordReaderStates::VarList => RecordReaderState::state_var_list(self, keyword),
            RecordReaderStates::SeqList => RecordReaderState::state_seq_list(self, keyword),
        }
    }

    fn state_header(mut self, keyword: Keywords) -> ReaderResult<Self> {
        match keyword {
            Keywords::CITIFile{version} => {
                match self.record.header.version {
                    Some(_) => Err(ReaderError::SingleUseKeywordDefinedTwice(Keywords::CITIFile{version})),
                    None => {
                        self.record.header.version = Some(version);
                        Ok(self)
                    }
                }
            },
            Keywords::Name(name) => {
                match self.record.header.name {
                    Some(_) => Err(ReaderError::SingleUseKeywordDefinedTwice(Keywords::Name(name))),
                    None => {
                        self.record.header.name = Some(name);
                        Ok(self)
                    }
                }
            },
            Keywords::Device{name, value} => {
                self.record.header.add_device(&name, &value);
                Ok(self)
            },
            Keywords::Comment(comment) => {
                self.record.header.comments.push(comment);
                Ok(self)
            },
            Keywords::Constant{name, value} => {
                self.record.header.constants.push(Constant::new(&name, &value));
                Ok(self)
            },
            Keywords::Var{name, format, length} => {
                match self.record.header.independent_variable.name {
                    Some(_) => Err(ReaderError::SingleUseKeywordDefinedTwice(Keywords::Var{name, format, length})),
                    None => {
                        self.record.header.independent_variable.name = Some(name);
                        self.record.header.independent_variable.format = format;
                        Ok(self)
                    },
                }
            },
            Keywords::VarListBegin => {
                match self.independent_variable_already_read {
                    false => {
                        self.state = RecordReaderStates::VarList;
                        Ok(self)
                    },
                    true => Err(ReaderError::IndependentVariableDefinedTwice),
                }
            },
            Keywords::SegListBegin => {
                match self.independent_variable_already_read {
                    false => {
                        self.state = RecordReaderStates::SeqList;
                        Ok(self)
                    },
                    true => Err(ReaderError::IndependentVariableDefinedTwice),
                }
            },
            Keywords::Begin => {
                self.state = RecordReaderStates::Data;
                Ok(self)
            },
            Keywords::Data{name, format} => {
                self.record.data.push(DataArray::new(&name, &format));
                Ok(self)
            },
            _ => Err(ReaderError::OutOfOrderKeyword(keyword)),
        }
    }

    fn state_data(mut self, keyword: Keywords) -> ReaderResult<Self> {
        match keyword {
            Keywords::DataPair{real, imag} => {
                if self.data_array_counter < self.record.data.len() {
                    self.record.data[self.data_array_counter].add_sample(real, imag);
                    Ok(self)
                } else {
                    Err(ReaderError::DataArrayOverIndex)
                }
            }
            Keywords::End => {
                self.state = RecordReaderStates::Header;
                self.data_array_counter += 1;
                Ok(self)
            },
            _ => Err(ReaderError::OutOfOrderKeyword(keyword)),
        }
    }

    fn state_var_list(mut self, keyword: Keywords) -> ReaderResult<Self> {
        match keyword {
            Keywords::VarListItem(value) => {
                self.record.header.independent_variable.push(value);
                Ok(self)
            },
            Keywords::VarListEnd => {
                self.independent_variable_already_read = true;
                self.state = RecordReaderStates::Header;
                Ok(self)
            },
            _ => Err(ReaderError::OutOfOrderKeyword(keyword)),
        }
    }

    fn state_seq_list(mut self, keyword: Keywords) -> ReaderResult<Self> {
        match keyword {
            Keywords::SegItem{first, last, number} => {
                self.record.header.independent_variable.seq(first, last, number);
                Ok(self)
            },
            Keywords::SegListEnd => {
                self.independent_variable_already_read = true;
                self.state = RecordReaderStates::Header;
                Ok(self)
            },
            _ => Err(ReaderError::OutOfOrderKeyword(keyword)),
        }
    }
}

#[cfg(test)]
mod test_record_reader_state {
    use super::*;
    use approx::*;

    #[test]
    fn test_new() {
        let expected = RecordReaderState{
            record: Record {
                header: Header {
                    version: None,
                    name: None,
                    comments: vec![],
                    devices: vec![],
                    independent_variable: Var {name: None, format: None, data: vec![]},
                    constants: vec![],
                },
                data: vec![],
            },
            state: RecordReaderStates::Header,
            data_array_counter: 0,
            independent_variable_already_read: false,
        };
        let result = RecordReaderState::new();
        assert_eq!(result, expected);
    }

    mod test_state_header {
        use super::*;

        mod test_keywords {
            use super::*;

            fn initialize_state() -> RecordReaderState {
                RecordReaderState{
                    state: RecordReaderStates::Header,
                    .. RecordReaderState::new()
                }
            }

            #[test]
            fn citirecord() {
                let keyword = Keywords::CITIFile{version: String::from("A.01.01")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.version, Some(String::from("A.01.01")));
                        assert_eq!(s.state, RecordReaderStates::Header);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn citirecord_cannot_be_called_twice() {
                let keyword = Keywords::CITIFile{version: String::from("A.01.01")};
                let mut state = initialize_state();
                state.record.header.version = Some(String::from("A.01.01"));
                match state.process_keyword(keyword) {
                    Err(ReaderError::SingleUseKeywordDefinedTwice(Keywords::CITIFile{version})) => assert_eq!(version, "A.01.01"),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn name() {
                let keyword = Keywords::Name(String::from("Name"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.name, Some(String::from("Name")));
                        assert_eq!(s.state, RecordReaderStates::Header);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn name_cannot_be_called_twice() {
                let keyword = Keywords::Name(String::from("CAL_SET"));
                let mut state = initialize_state();
                state.record.header.name = Some(String::from("CAL_SET"));
                match state.process_keyword(keyword) {
                    Err(ReaderError::SingleUseKeywordDefinedTwice(Keywords::Name(name))) => assert_eq!(name, "CAL_SET"),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_none() {
                let keyword = Keywords::Var{name: String::from("Name"), format: None, length: 102};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.independent_variable.name, Some(String::from("Name")));
                        assert_eq!(s.record.header.independent_variable.format, None);
                        assert_eq!(s.state, RecordReaderStates::Header);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_some() {
                let keyword = Keywords::Var{name: String::from("Name"), format: Some(String::from("MAG")), length: 102};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.independent_variable.name, Some(String::from("Name")));
                        assert_eq!(s.record.header.independent_variable.format, Some(String::from("MAG")));
                        assert_eq!(s.state, RecordReaderStates::Header);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_cannot_be_called_twice() {
                let keyword = Keywords::Var{name: String::from("FREQ"), format: None, length: 102};
                let mut state = initialize_state();
                state.record.header.independent_variable.name = Some(String::from("Name"));
                match state.process_keyword(keyword) {
                    Err(ReaderError::SingleUseKeywordDefinedTwice(Keywords::Var{name, format, length})) => {
                        assert_eq!(name, "FREQ");
                        assert_eq!(format, None);
                        assert_eq!(length, 102);
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_cannot_be_called_twice_none_format() {
                let keyword = Keywords::Var{name: String::from("FREQ"), format: Some(String::from("MAG")), length: 102};
                let mut state = initialize_state();
                state.record.header.independent_variable.name = Some(String::from("Name"));
                match state.process_keyword(keyword) {
                    Err(ReaderError::SingleUseKeywordDefinedTwice(Keywords::Var{name, format, length})) => {
                        assert_eq!(name, "FREQ");
                        assert_eq!(format, Some(String::from("MAG")));
                        assert_eq!(length, 102);
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn constant_empty() {
                let keyword = Keywords::Constant{name: String::from("Name"), value: String::from("Value")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.constants, vec![Constant::new("Name", "Value")]);
                        assert_eq!(s.state, RecordReaderStates::Header);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn constant_exists() {
                let keyword = Keywords::Constant{name: String::from("New Name"), value: String::from("New Value")};
                let mut state = initialize_state();
                state.record.header.constants.push(Constant::new("Name", "Value"));
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.constants, vec![Constant::new("Name", "Value"), Constant::new("New Name", "New Value")]);
                        assert_eq!(s.state, RecordReaderStates::Header);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn device() {
                let keyword = Keywords::Device{name: String::from("NA"), value: String::from("Value")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.devices.len(), 1);
                        assert_eq!(s.record.header.devices[0], Device{name: String::from("NA"), entries: vec![String::from("Value")]});
                        assert_eq!(s.state, RecordReaderStates::Header);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn device_with_existing_device() {
                let keyword = Keywords::Device{name: String::from("WVI"), value: String::from("1904")};
                let mut state = initialize_state();
                state.record.header.add_device("NA", "Value");
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.devices.len(), 2);
                        assert_eq!(s.record.header.devices[0], Device{name: String::from("NA"), entries: vec![String::from("Value")]});
                        assert_eq!(s.record.header.devices[1], Device{name: String::from("WVI"), entries: vec![String::from("1904")]});
                        assert_eq!(s.state, RecordReaderStates::Header);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_begin() {
                let keyword = Keywords::SegListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => assert_eq!(s.state, RecordReaderStates::SeqList),
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_begin_when_already_read() {
                let keyword = Keywords::SegListBegin;
                let mut state = initialize_state();
                state.independent_variable_already_read = true;
                match state.process_keyword(keyword) {
                    Err(ReaderError::IndependentVariableDefinedTwice) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_item() {
                let keyword = Keywords::SegItem{first: 10., last: 100., number: 2};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::SegItem{first, last, number})) => {
                        assert_relative_eq!(first, 10.);
                        assert_relative_eq!(last, 100.);
                        assert_eq!(number, 2);
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_end() {
                let keyword = Keywords::SegListEnd;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::SegListEnd)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_begin() {
                let keyword = Keywords::VarListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => assert_eq!(s.state, RecordReaderStates::VarList),
                    Err(e) => panic!("{:?}", e),
                }
            }
            
            #[test]
            fn var_list_begin_when_already_read() {
                let keyword = Keywords::VarListBegin;
                let mut state = initialize_state();
                state.independent_variable_already_read = true;
                match state.process_keyword(keyword) {
                    Err(ReaderError::IndependentVariableDefinedTwice) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_item() {
                let keyword = Keywords::VarListItem(1.);
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::VarListItem(f))) => assert_relative_eq!(f, 1.),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn data() {
                let keyword = Keywords::Data{name: String::from("S[1,1]"), format: String::from("RI")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.data, vec![DataArray {name: Some(String::from("S[1,1]")), format: Some(String::from("RI")), real: vec![], imag: vec![]}]);
                        assert_eq!(s.state, RecordReaderStates::Header);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn data_with_already_existing() {
                let keyword = Keywords::Data{name: String::from("S[1,1]"), format: String::from("RI")};
                let mut state = initialize_state();
                state.record.data.push(DataArray {name: Some(String::from("E")), format: Some(String::from("RI")), real: vec![], imag: vec![]});
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.data, vec![
                            DataArray {name: Some(String::from("E")), format: Some(String::from("RI")), real: vec![], imag: vec![]},
                            DataArray {name: Some(String::from("S[1,1]")), format: Some(String::from("RI")), real: vec![], imag: vec![]}
                        ]);
                        assert_eq!(s.state, RecordReaderStates::Header);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn data_pair() {
                let keyword = Keywords::DataPair{real: 1., imag: 2.};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::DataPair{real, imag})) => {
                        assert_relative_eq!(real, 1.);
                        assert_relative_eq!(imag, 2.);
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn begin() {
                let keyword = Keywords::Begin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.data_array_counter, 0);
                        assert_eq!(s.state, RecordReaderStates::Data);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn end() {
                let keyword = Keywords::End;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::End)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn comment() {
                let keyword = Keywords::Comment(String::from("Comment"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.comments, vec![String::from("Comment")]);
                        assert_eq!(s.state, RecordReaderStates::Header);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn comment_with_existing() {
                let keyword = Keywords::Comment(String::from("Comment"));
                let mut state = initialize_state();
                state.record.header.comments.push(String::from("Comment First"));
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.comments, vec![String::from("Comment First"), String::from("Comment")]);
                        assert_eq!(s.state, RecordReaderStates::Header);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }
        }   
    }

    mod test_state_data{
        use super::*;

        mod test_keywords {
            use super::*;

            fn initialize_state() -> RecordReaderState {
                let mut state = RecordReaderState{
                    state: RecordReaderStates::Data,
                    .. RecordReaderState::new()
                };
                state.record.data.push(DataArray::blank());
                state
            }

            #[test]
            fn citirecord() {
                let keyword = Keywords::CITIFile{version: String::from("A.01.01")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::CITIFile{version})) => assert_eq!(version, "A.01.01"),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn name() {
                let keyword = Keywords::Name(String::from("Name"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Name(name))) => assert_eq!(name, "Name"),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_none() {
                let keyword = Keywords::Var{name: String::from("Name"), format: None, length: 102};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Var{name, format, length})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, None);
                        assert_eq!(length, 102);
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_some() {
                let keyword = Keywords::Var{name: String::from("Name"), format: Some(String::from("MAG")), length: 102};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Var{name, format, length})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, Some(String::from("MAG")));
                        assert_eq!(length, 102);
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn constant() {
                let keyword = Keywords::Constant{name: String::from("Name"), value: String::from("Value")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Constant{name, value})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(value, "Value");
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn device() {
                let keyword = Keywords::Device{name: String::from("Name"), value: String::from("Value")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Device{name, value})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(value, "Value");
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_begin() {
                let keyword = Keywords::SegListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::SegListBegin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_item() {
                let keyword = Keywords::SegItem{first: 10., last: 100., number: 2};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::SegItem{first, last, number})) => {
                        assert_relative_eq!(first, 10.);
                        assert_relative_eq!(last, 100.);
                        assert_eq!(number, 2);
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_end() {
                let keyword = Keywords::SegListEnd;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::SegListEnd)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_begin() {
                let keyword = Keywords::VarListBegin;
                let state = initialize_state();
                match  state.process_keyword(keyword){ 
                    Err(ReaderError::OutOfOrderKeyword(Keywords::VarListBegin)) => (),
                    e => panic!("{:?}", e),
                }
            }
            
            #[test]
            fn var_list_item() {
                let keyword = Keywords::VarListItem(1.);
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::VarListItem(f))) => assert_relative_eq!(f, 1.),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_item_exponent() {
                let keyword = Keywords::VarListItem(1e9);
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::VarListItem(f))) => assert_relative_eq!(f, 1e9),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_end() {
                let keyword = Keywords::VarListEnd;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::VarListEnd)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn data() {
                let keyword = Keywords::Data{name: String::from("Name"), format: String::from("Format")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Data{name, format})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, "Format");
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn data_pair() {
                let keyword = Keywords::DataPair{real: 1., imag: 2.};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.data[0].real, vec![1.]);
                        assert_eq!(s.record.data[0].imag, vec![2.]);
                        assert_eq!(s.state, RecordReaderStates::Data);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn data_pair_second_array() {
                let keyword = Keywords::DataPair{real: 1., imag: 2.};
                let mut state = initialize_state();
                state.record.data.push(DataArray::blank());
                state.data_array_counter = 1;
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.data[0].real, vec![]);
                        assert_eq!(s.record.data[0].imag, vec![]);
                        assert_eq!(s.record.data[1].real, vec![1.]);
                        assert_eq!(s.record.data[1].imag, vec![2.]);
                        assert_eq!(s.state, RecordReaderStates::Data);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn data_pair_out_of_bounds() {
                let keyword = Keywords::DataPair{real: 1., imag: 2.};
                let mut state = initialize_state();
                state.data_array_counter = 1;
                match state.process_keyword(keyword) {
                    Err(ReaderError::DataArrayOverIndex) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn begin() {
                let keyword = Keywords::Begin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Begin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn end() {
                let keyword = Keywords::End;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.state, RecordReaderStates::Header);
                        assert_eq!(s.data_array_counter, 1);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn end_increment_index() {
                let keyword = Keywords::End;
                let mut state = initialize_state();
                state.data_array_counter = 1;
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.state, RecordReaderStates::Header);
                        assert_eq!(s.data_array_counter, 2);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn comment() {
                let keyword = Keywords::Comment(String::from("Comment"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Comment(comment))) => assert_eq!(comment, "Comment"),
                    e => panic!("{:?}", e),
                }
            }
        }   
    }

    mod test_state_var_list{
        use super::*;

        mod test_keywords {
            use super::*;

            fn initialize_state() -> RecordReaderState {
                RecordReaderState{
                    state: RecordReaderStates::VarList,
                    .. RecordReaderState::new()
                }
            }

            #[test]
            fn citirecord() {
                let keyword = Keywords::CITIFile{version: String::from("A.01.01")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::CITIFile{version})) => assert_eq!(version, "A.01.01"),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn name() {
                let keyword = Keywords::Name(String::from("Name"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Name(name))) => assert_eq!(name, "Name"),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_none() {
                let keyword = Keywords::Var{name: String::from("Name"), format: None, length: 102};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Var{name, format, length})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, None);
                        assert_eq!(length, 102);
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_some() {
                let keyword = Keywords::Var{name: String::from("Name"), format: Some(String::from("MAG")), length: 102};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Var{name, format, length})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, Some(String::from("MAG")));
                        assert_eq!(length, 102);
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn constant() {
                let keyword = Keywords::Constant{name: String::from("Name"), value: String::from("Value")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Constant{name, value})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(value, "Value");
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn device() {
                let keyword = Keywords::Device{name: String::from("Name"), value: String::from("Value")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Device{name, value})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(value, "Value");
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_begin() {
                let keyword = Keywords::SegListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::SegListBegin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_item() {
                let keyword = Keywords::SegItem{first: 10., last: 100., number: 2};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::SegItem{first, last, number})) => {
                        assert_relative_eq!(first, 10.);
                        assert_relative_eq!(last, 100.);
                        assert_eq!(number, 2);
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_end() {
                let keyword = Keywords::SegListEnd;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::SegListEnd)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_begin() {
                let keyword = Keywords::VarListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::VarListBegin)) => (),
                    e => panic!("{:?}", e),
                }
            }
            
            #[test]
            fn var_list_item() {
                let keyword = Keywords::VarListItem(1.);
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.independent_variable.data, vec![1.]);
                        assert_eq!(s.state, RecordReaderStates::VarList);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_item_exponent() {
                let keyword = Keywords::VarListItem(1e9);
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.independent_variable.data, vec![1e9]);
                        assert_eq!(s.state, RecordReaderStates::VarList);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_item_already_exists() {
                let keyword = Keywords::VarListItem(1e9);
                let mut state = initialize_state();
                state.record.header.independent_variable.push(1e8);
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.independent_variable.data, vec![1e8, 1e9]);
                        assert_eq!(s.state, RecordReaderStates::VarList);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_end() {
                let keyword = Keywords::VarListEnd;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.independent_variable_already_read, true);
                        assert_eq!(s.state, RecordReaderStates::Header);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn data() {
                let keyword = Keywords::Data{name: String::from("Name"), format: String::from("Format")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Data{name, format})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, "Format");
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn data_pair() {
                let keyword = Keywords::DataPair{real: 1., imag: 1.};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::DataPair{real, imag})) => {
                        assert_relative_eq!(real, 1.);
                        assert_relative_eq!(imag, 1.);
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn begin() {
                let keyword = Keywords::Begin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Begin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn end() {
                let keyword = Keywords::End;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::End)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn comment() {
                let keyword = Keywords::Comment(String::from("Comment"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Comment(comment))) => assert_eq!(comment, "Comment"),
                    e => panic!("{:?}", e),
                }
            }
        }   
    }

    mod test_state_seq_list{
        use super::*;

        mod test_keywords {
            use super::*;

            fn initialize_state() -> RecordReaderState {
                RecordReaderState{
                    state: RecordReaderStates::SeqList,
                    .. RecordReaderState::new()
                }
            }

            #[test]
            fn citirecord() {
                let keyword = Keywords::CITIFile{version: String::from("A.01.01")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::CITIFile{version})) => assert_eq!(version, "A.01.01"),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn name() {
                let keyword = Keywords::Name(String::from("Name"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Name(name))) => assert_eq!(name, "Name"),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_none() {
                let keyword = Keywords::Var{name: String::from("Name"), format: None, length: 102};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Var{name, format, length})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, None);
                        assert_eq!(length, 102);
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_some() {
                let keyword = Keywords::Var{name: String::from("Name"), format: Some(String::from("MAG")), length: 102};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Var{name, format, length})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, Some(String::from("MAG")));
                        assert_eq!(length, 102);
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn constant() {
                let keyword = Keywords::Constant{name: String::from("Name"), value: String::from("Value")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Constant{name, value})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(value, "Value");
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn device() {
                let keyword = Keywords::Device{name: String::from("Name"), value: String::from("Value")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Device{name, value})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(value, "Value");
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_begin() {
                let keyword = Keywords::SegListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::SegListBegin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_item() {
                let keyword = Keywords::SegItem{first: 10., last: 100., number: 2};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.independent_variable.data, vec![10., 100.]);
                        assert_eq!(s.state, RecordReaderStates::SeqList);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_item_triple() {
                let keyword = Keywords::SegItem{first: 10., last: 100., number: 3};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.independent_variable.data, vec![10., 55., 100.]);
                        assert_eq!(s.state, RecordReaderStates::SeqList);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_end() {
                let keyword = Keywords::SegListEnd;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.independent_variable_already_read, true);
                        assert_eq!(s.state, RecordReaderStates::Header);
                    },
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_begin() {
                let keyword = Keywords::VarListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::VarListBegin)) => (),
                    e => panic!("{:?}", e),
                }
            }
            
            #[test]
            fn var_list_item() {
                let keyword = Keywords::VarListItem(1.);
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::VarListItem(f))) => assert_relative_eq!(f, 1.0),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_end() {
                let keyword = Keywords::VarListEnd;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::VarListEnd)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn data() {
                let keyword = Keywords::Data{name: String::from("Name"), format: String::from("Format")};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Data{name, format})) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, "Format");
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn data_pair() {
                let keyword = Keywords::DataPair{real: 1., imag: 1.};
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::DataPair{real, imag})) => {
                        assert_relative_eq!(real, 1.);
                        assert_relative_eq!(imag, 1.);
                    },
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn begin() {
                let keyword = Keywords::Begin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Begin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn end() {
                let keyword = Keywords::End;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::End)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn comment() {
                let keyword = Keywords::Comment(String::from("Comment"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReaderError::OutOfOrderKeyword(Keywords::Comment(s))) => assert_eq!(s, "Comment"),
                    e => panic!("{:?}", e),
                }
            }
        }   
    }
}
