//! Input/Output for CITI records
//!
//! <p><a href="http://literature.cdn.keysight.com/litweb/pdf/ads15/cktsim/ck2016.html#:~:text=CITIrecord%20stands%20for%20Common%20Instrumentation,it%20can%20meet%20future%20needs">The standard</a>, defines the following entities:</p>
//!
//! | Name     | Description                       |
//! |----------|-----------------------------------|
//! | Record   | The entire contents of the record |
//! | Header   | Header of the record              |
//! | Data     | One or more data arrays           |
//! | Keyword  | Define the header contents        |
//!
//! As this is a custom ASCII record type, the standard is not as simple as one would like.
//! The standard is followed as closely as is reasonable. The largest changes are in the
//! extension of the keywords.
//!
//! ## Non-Standard Type
//!
//! A non-standard but industry prevelent comment section is added formated with a bang:
//!
//! ```no_test
//! !COMMENT
//! ```
//!
//! These are used to provide internal comments.
//!
//! ## IO Example
//!
//! The object must implement the [`BufRead`] trait since CITI files are read line-by-line.
//! As a result, two reads will lead to a fail on the second read, since the buffer is empty.
//!
//! Read file:
//! ```no_run
//! use citi::Record;
//! use std::fs::File;
//!
//! let mut file = File::open("file.cti").unwrap();
//! let record = Record::from_reader(&mut file);
//! ```
//!
//! Write file:
//! ```no_run
//! use citi::Record;
//! use std::fs::File;
//!
//! let record = Record::default();
//! let mut file = File::create("file.cti").unwrap();
//! record.to_writer(&mut file);
//! ```
//!
//! ## Input-Output Consistency:
//!
//! General input-output consistency cannot be guaranteed with CITI records because of their
//! design. That is, if a record is read in and read out, the byte representation of the record
//! may change, exact floating point representations may change, but the record will contain the
//! same information. The following is not guaranteed:
//!
//! - ASCII representation of floating points may change because of the String -> Float -> String conversion.
//! - Floats may be shifted in exponential format.
//! - All `SEG_LIST` keywords will be converted to `VAR_LIST`

use lazy_static::lazy_static;
use num_complex::Complex;
use regex::Regex;

use std::convert::TryFrom;
use std::fmt;
use std::io::BufRead;
use std::str::FromStr;

use thiserror::Error;

mod macros;
pub mod ffi;

/// Crate error
///
/// This is the highest level error in this crate. No
/// other errors need to be explicitly dealt with.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Parsing error: `{0}`")]
    ParseError(#[from] ParseError),
    #[error("Reading error: `{0}`")]
    ReadError(#[from] ReadError),
    #[error("Error writing record: `{0}`")]
    WriteError(#[from] WriteError),
}
/// Crate interface result
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod test_error {
    use super::*;

    mod test_display {
        use super::*;

        #[test]
        fn parse_error() {
            let error = Error::ParseError(ParseError::BadRegex);
            assert_eq!(
                format!("{}", error),
                "Parsing error: `Regex could not be parsed`"
            );
        }

        #[test]
        fn reader_error() {
            let error = Error::ReadError(ReadError::DataArrayOverIndex);
            assert_eq!(
                format!("{}", error),
                "Reading error: `More data arrays than defined in header`"
            );
        }

        #[test]
        fn write_error() {
            let error = Error::WriteError(WriteError::NoVersion);
            assert_eq!(
                format!("{}", error),
                "Error writing record: `Version is not defined`"
            );
        }
    }

    mod from_error {
        use super::*;

        #[test]
        fn from_parse_error() {
            match Error::from(ParseError::BadRegex) {
                Error::ParseError(ParseError::BadRegex) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn from_reader_error() {
            match Error::from(ReadError::DataArrayOverIndex) {
                Error::ReadError(ReadError::DataArrayOverIndex) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn from_write_error() {
            match Error::from(WriteError::NoVersion) {
                Error::WriteError(WriteError::NoVersion) => (),
                e => panic!("{:?}", e),
            }
        }
    }
}

/// Error from parsing a line
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

/// Representation of the per-line keywords
///
/// A vector of these keywords represents a file.
#[derive(Debug, PartialEq)]
pub enum Keyword {
    /// CitiFile version e.g. A.01.01
    CitiFile { version: String },
    /// Name. Single word with no spaces. e.g. CAL_SET
    Name(String),
    /// Independent variable with name, format, and number of samples. e.g. VAR
    /// FREQ MAG 201
    Var {
        name: String,
        format: String,
        length: usize,
    },
    /// Constant with name and value. e.g. CONSTANT A A_THING
    Constant { name: String, value: String },
    /// New Device
    Device { name: String, value: String },
    /// Beginning of independent variable segments
    SegListBegin,
    /// An item in a SEG list
    SegItem {
        first: f64,
        last: f64,
        number: usize,
    },
    /// End of independent variable segments
    SegListEnd,
    /// Beginning of independent variable list
    VarListBegin,
    /// Item in a var list
    VarListItem(f64),
    /// End of independent variable list
    VarListEnd,
    /// Define a data array. e.g. DATA S\[1,1\] RI
    Data { name: String, format: String },
    /// Real, Imaginary pair in data
    DataPair { real: f64, imag: f64 },
    /// Begin data array
    Begin,
    /// End data array
    End,
    /// Comment (non-standard)
    Comment(String),
}

impl FromStr for Keyword {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Keyword::try_from(s)
    }
}

impl TryFrom<&str> for Keyword {
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
            "SEG_LIST_BEGIN" => Ok(Keyword::SegListBegin),
            "SEG_LIST_END" => Ok(Keyword::SegListEnd),
            "VAR_LIST_BEGIN" => Ok(Keyword::VarListBegin),
            "VAR_LIST_END" => Ok(Keyword::VarListEnd),
            "BEGIN" => Ok(Keyword::Begin),
            "END" => Ok(Keyword::End),
            _ if RE_DATA_PAIR.is_match(line) => {
                let cap = RE_DATA_PAIR.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keyword::DataPair {
                    real: cap
                        .name("Real")
                        .map(|m| m.as_str())
                        .ok_or(ParseError::BadRegex)?
                        .parse::<f64>()
                        .map_err(|_| ParseError::NumberParseError(String::from(line)))?,
                    imag: cap
                        .name("Imag")
                        .map(|m| m.as_str())
                        .ok_or(ParseError::BadRegex)?
                        .parse::<f64>()
                        .map_err(|_| ParseError::NumberParseError(String::from(line)))?,
                })
            }
            _ if RE_DEVICE.is_match(line) => {
                let cap = RE_DEVICE.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keyword::Device {
                    name: String::from(
                        cap.name("Name")
                            .map(|m| m.as_str())
                            .ok_or(ParseError::BadRegex)?,
                    ),
                    value: String::from(
                        cap.name("Value")
                            .map(|m| m.as_str())
                            .ok_or(ParseError::BadRegex)?,
                    ),
                })
            }
            _ if RE_SEG_ITEM.is_match(line) => {
                let cap = RE_SEG_ITEM.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keyword::SegItem {
                    first: cap
                        .name("First")
                        .map(|m| m.as_str())
                        .ok_or(ParseError::BadRegex)?
                        .parse::<f64>()
                        .map_err(|_| ParseError::NumberParseError(String::from(line)))?,
                    last: cap
                        .name("Last")
                        .map(|m| m.as_str())
                        .ok_or(ParseError::BadRegex)?
                        .parse::<f64>()
                        .map_err(|_| ParseError::NumberParseError(String::from(line)))?,
                    number: cap
                        .name("Number")
                        .map(|m| m.as_str())
                        .ok_or(ParseError::BadRegex)?
                        .parse::<usize>()
                        .map_err(|_| ParseError::NumberParseError(String::from(line)))?,
                })
            }
            _ if RE_VAR_ITEM.is_match(line) => {
                let cap = RE_VAR_ITEM.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keyword::VarListItem(
                    cap.name("Value")
                        .map(|m| m.as_str())
                        .ok_or(ParseError::BadRegex)?
                        .parse::<f64>()
                        .map_err(|_| ParseError::NumberParseError(String::from(line)))?,
                ))
            }
            _ if RE_DATA.is_match(line) => {
                let cap = RE_DATA.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keyword::Data {
                    name: String::from(
                        cap.name("Name")
                            .map(|m| m.as_str())
                            .ok_or(ParseError::BadRegex)?,
                    ),
                    format: String::from(
                        cap.name("Format")
                            .map(|m| m.as_str())
                            .ok_or(ParseError::BadRegex)?,
                    ),
                })
            }
            _ if RE_VAR.is_match(line) => {
                let cap = RE_VAR.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keyword::Var {
                    name: String::from(
                        cap.name("Name")
                            .map(|m| m.as_str())
                            .ok_or(ParseError::BadRegex)?,
                    ),
                    format: String::from(
                        cap.name("Format")
                            .map(|m| m.as_str())
                            .ok_or(ParseError::BadRegex)?,
                    ),
                    length: cap
                        .name("Length")
                        .map(|m| m.as_str())
                        .ok_or(ParseError::BadRegex)?
                        .parse::<usize>()
                        .map_err(|_| ParseError::NumberParseError(String::from(line)))?,
                })
            }
            _ if RE_COMMENT.is_match(line) => {
                let cap = RE_COMMENT.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keyword::Comment(String::from(
                    cap.name("Comment")
                        .map(|m| m.as_str())
                        .ok_or(ParseError::BadRegex)?,
                )))
            }
            _ if RE_CITIFILE.is_match(line) => {
                let cap = RE_CITIFILE.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keyword::CitiFile {
                    version: String::from(
                        cap.name("Version")
                            .map(|m| m.as_str())
                            .ok_or(ParseError::BadRegex)?,
                    ),
                })
            }
            _ if RE_NAME.is_match(line) => {
                let cap = RE_NAME.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keyword::Name(String::from(
                    cap.name("Name")
                        .map(|m| m.as_str())
                        .ok_or(ParseError::BadRegex)?,
                )))
            }
            _ if RE_CONSTANT.is_match(line) => {
                let cap = RE_CONSTANT.captures(line).ok_or(ParseError::BadRegex)?;
                Ok(Keyword::Constant {
                    name: String::from(
                        cap.name("Name")
                            .map(|m| m.as_str())
                            .ok_or(ParseError::BadRegex)?,
                    ),
                    value: String::from(
                        cap.name("Value")
                            .map(|m| m.as_str())
                            .ok_or(ParseError::BadRegex)?,
                    ),
                })
            }
            _ => Err(ParseError::BadKeyword(String::from(line))),
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Keyword::CitiFile { version } => write!(f, "CITIFILE {}", version),
            Keyword::Name(name) => write!(f, "NAME {}", name),
            Keyword::Var {
                name,
                format,
                length,
            } => write!(f, "VAR {} {} {}", name, format, length),
            Keyword::Constant { name, value } => write!(f, "CONSTANT {} {}", name, value),
            Keyword::Device { name, value } => write!(f, "#{} {}", name, value),
            Keyword::SegListBegin => write!(f, "SEG_LIST_BEGIN"),
            Keyword::SegItem {
                first,
                last,
                number,
            } => write!(f, "SEG {} {} {}", first, last, number),
            Keyword::SegListEnd => write!(f, "SEG_LIST_END"),
            Keyword::VarListBegin => write!(f, "VAR_LIST_BEGIN"),
            Keyword::VarListItem(n) => write!(f, "{}", n),
            Keyword::VarListEnd => write!(f, "VAR_LIST_END"),
            Keyword::Data { name, format } => write!(f, "DATA {} {}", name, format),
            Keyword::DataPair { real, imag } => write!(f, "{:E},{:E}", real, imag),
            Keyword::Begin => write!(f, "BEGIN"),
            Keyword::End => write!(f, "END"),
            Keyword::Comment(comment) => write!(f, "!{}", comment),
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
            let keyword = Keyword::CitiFile {
                version: String::from("A.01.00"),
            };
            assert_eq!("CITIFILE A.01.00", format!("{}", keyword));
        }

        #[test]
        fn citirecord_a_01_01() {
            let keyword = Keyword::CitiFile {
                version: String::from("A.01.01"),
            };
            assert_eq!("CITIFILE A.01.01", format!("{}", keyword));
        }

        #[test]
        fn name() {
            let keyword = Keyword::Name(String::from("CAL_SET"));
            assert_eq!("NAME CAL_SET", format!("{}", keyword));
        }

        #[test]
        fn var() {
            let keyword = Keyword::Var {
                name: String::from("FREQ"),
                format: String::from("MAG"),
                length: 201,
            };
            assert_eq!("VAR FREQ MAG 201", format!("{}", keyword));
        }

        #[test]
        fn constant() {
            let keyword = Keyword::Constant {
                name: String::from("A_CONSTANT"),
                value: String::from("1.2345"),
            };
            assert_eq!("CONSTANT A_CONSTANT 1.2345", format!("{}", keyword));
        }

        #[test]
        fn device() {
            let keyword = Keyword::Device {
                name: String::from("NA"),
                value: String::from("REGISTER 1"),
            };
            assert_eq!("#NA REGISTER 1", format!("{}", keyword));
        }

        #[test]
        fn device_number() {
            let keyword = Keyword::Device {
                name: String::from("NA"),
                value: String::from("POWER2 1.0E1"),
            };
            assert_eq!("#NA POWER2 1.0E1", format!("{}", keyword));
        }

        #[test]
        fn device_name() {
            let keyword = Keyword::Device {
                name: String::from("WVI"),
                value: String::from("A B"),
            };
            assert_eq!("#WVI A B", format!("{}", keyword));
        }

        #[test]
        fn seg_list_begin() {
            let keyword = Keyword::SegListBegin;
            assert_eq!("SEG_LIST_BEGIN", format!("{}", keyword));
        }

        #[test]
        fn seg_item() {
            let keyword = Keyword::SegItem {
                first: 1000000000.,
                last: 4000000000.,
                number: 10,
            };
            assert_eq!("SEG 1000000000 4000000000 10", format!("{}", keyword));
        }

        #[test]
        fn seg_list_end() {
            let keyword = Keyword::SegListEnd;
            assert_eq!("SEG_LIST_END", format!("{}", keyword));
        }

        #[test]
        fn var_list_begin() {
            let keyword = Keyword::VarListBegin;
            assert_eq!("VAR_LIST_BEGIN", format!("{}", keyword));
        }

        #[test]
        fn var_item() {
            let keyword = Keyword::VarListItem(100000.);
            assert_eq!("100000", format!("{}", keyword));
        }

        #[test]
        fn var_list_end() {
            let keyword = Keyword::VarListEnd;
            assert_eq!("VAR_LIST_END", format!("{}", keyword));
        }

        #[test]
        fn data_s11() {
            let keyword = Keyword::Data {
                name: String::from("S[1,1]"),
                format: String::from("RI"),
            };
            assert_eq!("DATA S[1,1] RI", format!("{}", keyword));
        }

        #[test]
        fn data_e() {
            let keyword = Keyword::Data {
                name: String::from("E"),
                format: String::from("RI"),
            };
            assert_eq!("DATA E RI", format!("{}", keyword));
        }

        #[test]
        fn data_pair_simple() {
            let keyword = Keyword::DataPair {
                real: 1e9,
                imag: -1e9,
            };
            assert_eq!("1E9,-1E9", format!("{}", keyword));
        }

        #[test]
        fn data_pair() {
            let keyword = Keyword::DataPair {
                real: 0.86303e-1,
                imag: -8.98651e-1,
            };
            assert_eq!("8.6303E-2,-8.98651E-1", format!("{}", keyword));
        }

        #[test]
        fn begin() {
            let keyword = Keyword::Begin;
            assert_eq!("BEGIN", format!("{}", keyword));
        }

        #[test]
        fn end() {
            let keyword = Keyword::End;
            assert_eq!("END", format!("{}", keyword));
        }

        #[test]
        fn comment() {
            let keyword = Keyword::Comment(String::from("DATE: 2019.11.01"));
            assert_eq!("!DATE: 2019.11.01", format!("{}", keyword));
        }
    }

    #[cfg(test)]
    mod test_from_str_slice {
        use super::*;
        use approx::*;

        #[test]
        fn fails_on_bad_string() {
            match Keyword::from_str("this is a bad string") {
                Err(ParseError::BadKeyword(bad_string)) => {
                    assert_eq!(bad_string, "this is a bad string")
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn citirecord_a_01_00() {
            match Keyword::from_str("CITIFILE A.01.00") {
                Ok(Keyword::CitiFile { version }) => assert_eq!(version, "A.01.00"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn citirecord_a_01_01() {
            match Keyword::from_str("CITIFILE A.01.01") {
                Ok(Keyword::CitiFile { version }) => assert_eq!(version, "A.01.01"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn name_cal_set() {
            match Keyword::from_str("NAME CAL_SET") {
                Ok(Keyword::Name(name)) => assert_eq!(name, "CAL_SET"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn name_raw_data() {
            match Keyword::from_str("NAME RAW_DATA") {
                Ok(Keyword::Name(name)) => assert_eq!(name, "RAW_DATA"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn constant() {
            match Keyword::from_str("CONSTANT A_CONSTANT 1.2345") {
                Ok(Keyword::Constant { name, value }) => {
                    assert_eq!(name, "A_CONSTANT");
                    assert_eq!(value, "1.2345");
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn device() {
            match Keyword::from_str("#NA REGISTER 1") {
                Ok(Keyword::Device { name, value }) => {
                    assert_eq!(name, "NA");
                    assert_eq!(value, "REGISTER 1");
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn device_number() {
            match Keyword::from_str("#NA POWER2 1.0E1") {
                Ok(Keyword::Device { name, value }) => {
                    assert_eq!(name, "NA");
                    assert_eq!(value, "POWER2 1.0E1");
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn device_name() {
            match Keyword::from_str("#WVI A B") {
                Ok(Keyword::Device { name, value }) => {
                    assert_eq!(name, "WVI");
                    assert_eq!(value, "A B");
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var() {
            match Keyword::from_str("VAR FREQ MAG 201") {
                Ok(Keyword::Var {
                    name,
                    format,
                    length,
                }) => {
                    assert_eq!(name, "FREQ");
                    assert_eq!(format, "MAG");
                    assert_eq!(length, 201);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_list_begin() {
            match Keyword::from_str("SEG_LIST_BEGIN") {
                Ok(Keyword::SegListBegin) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_item() {
            match Keyword::from_str("SEG 1000000000 4000000000 10") {
                Ok(Keyword::SegItem {
                    first,
                    last,
                    number,
                }) => {
                    assert_relative_eq!(first, 1000000000.);
                    assert_relative_eq!(last, 4000000000.);
                    assert_eq!(number, 10);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_item_exponential() {
            match Keyword::from_str("SEG 1e9 1E4 100") {
                Ok(Keyword::SegItem {
                    first,
                    last,
                    number,
                }) => {
                    assert_relative_eq!(first, 1e9);
                    assert_relative_eq!(last, 1e4);
                    assert_eq!(number, 100);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_item_negative() {
            match Keyword::from_str("SEG -1e9 1E-4 1") {
                Ok(Keyword::SegItem {
                    first,
                    last,
                    number,
                }) => {
                    assert_relative_eq!(first, -1e9);
                    assert_relative_eq!(last, 1e-4);
                    assert_eq!(number, 1);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_list_end() {
            match Keyword::from_str("SEG_LIST_END") {
                Ok(Keyword::SegListEnd) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_list_begin() {
            match Keyword::from_str("VAR_LIST_BEGIN") {
                Ok(Keyword::VarListBegin) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item() {
            match Keyword::from_str("100000") {
                Ok(Keyword::VarListItem(value)) => {
                    assert_relative_eq!(value, 100000.);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item_exponential() {
            match Keyword::from_str("100E+6") {
                Ok(Keyword::VarListItem(value)) => {
                    assert_relative_eq!(value, 100E+6);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item_negative_exponential() {
            match Keyword::from_str("-1e-2") {
                Ok(Keyword::VarListItem(value)) => {
                    assert_relative_eq!(value, -1e-2);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item_negative() {
            match Keyword::from_str("-100000") {
                Ok(Keyword::VarListItem(value)) => {
                    assert_relative_eq!(value, -100000.);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_list_end() {
            match Keyword::from_str("VAR_LIST_END") {
                Ok(Keyword::VarListEnd) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_s11() {
            match Keyword::from_str("DATA S[1,1] RI") {
                Ok(Keyword::Data { name, format }) => {
                    assert_eq!(name, "S[1,1]");
                    assert_eq!(format, "RI");
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_e() {
            match Keyword::from_str("DATA E RI") {
                Ok(Keyword::Data { name, format }) => {
                    assert_eq!(name, "E");
                    assert_eq!(format, "RI");
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_pair_simple() {
            match Keyword::from_str("1E9,-1E9") {
                Ok(Keyword::DataPair { real, imag }) => {
                    assert_relative_eq!(real, 1e9);
                    assert_relative_eq!(imag, -1e9);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_pair() {
            match Keyword::from_str("8.6303E-2,-8.98651E-1") {
                Ok(Keyword::DataPair { real, imag }) => {
                    assert_relative_eq!(real, 0.86303e-1);
                    assert_relative_eq!(imag, -8.98651e-1);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_pair_spaced() {
            match Keyword::from_str("8.6303E-2, -8.98651E-1") {
                Ok(Keyword::DataPair { real, imag }) => {
                    assert_relative_eq!(real, 0.86303e-1);
                    assert_relative_eq!(imag, -8.98651e-1);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn begin() {
            match Keyword::from_str("BEGIN") {
                Ok(Keyword::Begin) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn end() {
            match Keyword::from_str("END") {
                Ok(Keyword::End) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn comment() {
            match Keyword::from_str("!DATE: 2019.11.01") {
                Ok(Keyword::Comment(s)) => assert_eq!(s, "DATE: 2019.11.01"),
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
            match Keyword::try_from("this is a bad string") {
                Err(ParseError::BadKeyword(bad_string)) => {
                    assert_eq!(bad_string, "this is a bad string")
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn citirecord_a_01_00() {
            match Keyword::try_from("CITIFILE A.01.00") {
                Ok(Keyword::CitiFile { version }) => assert_eq!(version, "A.01.00"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn citirecord_a_01_01() {
            match Keyword::try_from("CITIFILE A.01.01") {
                Ok(Keyword::CitiFile { version }) => assert_eq!(version, "A.01.01"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn name_cal_set() {
            match Keyword::try_from("NAME CAL_SET") {
                Ok(Keyword::Name(name)) => assert_eq!(name, "CAL_SET"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn name_raw_data() {
            match Keyword::try_from("NAME RAW_DATA") {
                Ok(Keyword::Name(name)) => assert_eq!(name, "RAW_DATA"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn constant() {
            match Keyword::try_from("CONSTANT A_CONSTANT 1.2345") {
                Ok(Keyword::Constant { name, value }) => {
                    assert_eq!(name, "A_CONSTANT");
                    assert_eq!(value, "1.2345");
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn device() {
            match Keyword::try_from("#NA REGISTER 1") {
                Ok(Keyword::Device { name, value }) => {
                    assert_eq!(name, "NA");
                    assert_eq!(value, "REGISTER 1");
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn device_number() {
            match Keyword::try_from("#NA POWER2 1.0E1") {
                Ok(Keyword::Device { name, value }) => {
                    assert_eq!(name, "NA");
                    assert_eq!(value, "POWER2 1.0E1");
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn device_name() {
            match Keyword::try_from("#WVI A B") {
                Ok(Keyword::Device { name, value }) => {
                    assert_eq!(name, "WVI");
                    assert_eq!(value, "A B");
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var() {
            match Keyword::try_from("VAR FREQ MAG 201") {
                Ok(Keyword::Var {
                    name,
                    format,
                    length,
                }) => {
                    assert_eq!(name, "FREQ");
                    assert_eq!(format, "MAG");
                    assert_eq!(length, 201);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_list_begin() {
            match Keyword::try_from("SEG_LIST_BEGIN") {
                Ok(Keyword::SegListBegin) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_item() {
            match Keyword::try_from("SEG 1000000000 4000000000 10") {
                Ok(Keyword::SegItem {
                    first,
                    last,
                    number,
                }) => {
                    assert_relative_eq!(first, 1000000000.);
                    assert_relative_eq!(last, 4000000000.);
                    assert_eq!(number, 10);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_item_exponential() {
            match Keyword::try_from("SEG 1e9 1E4 100") {
                Ok(Keyword::SegItem {
                    first,
                    last,
                    number,
                }) => {
                    assert_relative_eq!(first, 1e9);
                    assert_relative_eq!(last, 1e4);
                    assert_eq!(number, 100);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_item_negative() {
            match Keyword::try_from("SEG -1e9 1E-4 1") {
                Ok(Keyword::SegItem {
                    first,
                    last,
                    number,
                }) => {
                    assert_relative_eq!(first, -1e9);
                    assert_relative_eq!(last, 1e-4);
                    assert_eq!(number, 1);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn seg_list_end() {
            match Keyword::try_from("SEG_LIST_END") {
                Ok(Keyword::SegListEnd) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_list_begin() {
            match Keyword::try_from("VAR_LIST_BEGIN") {
                Ok(Keyword::VarListBegin) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item() {
            match Keyword::try_from("100000") {
                Ok(Keyword::VarListItem(value)) => {
                    assert_relative_eq!(value, 100000.);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item_exponential() {
            match Keyword::try_from("100E+6") {
                Ok(Keyword::VarListItem(value)) => {
                    assert_relative_eq!(value, 100E+6);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item_negative_exponential() {
            match Keyword::try_from("-1e-2") {
                Ok(Keyword::VarListItem(value)) => {
                    assert_relative_eq!(value, -1e-2);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_item_negative() {
            match Keyword::try_from("-100000") {
                Ok(Keyword::VarListItem(value)) => {
                    assert_relative_eq!(value, -100000.);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn var_list_end() {
            match Keyword::try_from("VAR_LIST_END") {
                Ok(Keyword::VarListEnd) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_s11() {
            match Keyword::try_from("DATA S[1,1] RI") {
                Ok(Keyword::Data { name, format }) => {
                    assert_eq!(name, "S[1,1]");
                    assert_eq!(format, "RI");
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_e() {
            match Keyword::try_from("DATA E RI") {
                Ok(Keyword::Data { name, format }) => {
                    assert_eq!(name, "E");
                    assert_eq!(format, "RI");
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_pair_simple() {
            match Keyword::try_from("1E9,-1E9") {
                Ok(Keyword::DataPair { real, imag }) => {
                    assert_relative_eq!(real, 1e9);
                    assert_relative_eq!(imag, -1e9);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_pair() {
            match Keyword::try_from("8.6303E-2,-8.98651E-1") {
                Ok(Keyword::DataPair { real, imag }) => {
                    assert_relative_eq!(real, 0.86303e-1);
                    assert_relative_eq!(imag, -8.98651e-1);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data_pair_spaced() {
            match Keyword::try_from("8.6303E-2, -8.98651E-1") {
                Ok(Keyword::DataPair { real, imag }) => {
                    assert_relative_eq!(real, 0.86303e-1);
                    assert_relative_eq!(imag, -8.98651e-1);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn begin() {
            match Keyword::try_from("BEGIN") {
                Ok(Keyword::Begin) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn end() {
            match Keyword::try_from("END") {
                Ok(Keyword::End) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn comment() {
            match Keyword::try_from("!DATE: 2019.11.01") {
                Ok(Keyword::Comment(s)) => assert_eq!(s, "DATE: 2019.11.01"),
                e => panic!("{:?}", e),
            }
        }
    }
}

/// Device-specific value.
///
/// This should be used over constants to conform to the standard.
/// ```no_test
/// #NA VERSION HP8510B.05.00
/// ```
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
        let expected = Device {
            name: String::from("A Name"),
            entries: vec![],
        };
        assert_eq!(result, expected);
    }
}

/// The independent variable
#[derive(Debug, PartialEq, Clone)]
pub struct Var {
    pub name: String,
    pub format: String,
    pub data: Vec<f64>,
}

impl Var {
    fn blank() -> Var {
        Var {
            name: String::new(),
            format: String::new(),
            data: vec![],
        }
    }

    pub fn new(name: &str, format: &str) -> Var {
        Var {
            name: String::from(name),
            format: String::from(format),
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
                let delta = (last - first) / ((number - 1) as f64);
                for i in 0..number {
                    self.push(first + (i as f64) * delta);
                }
            }
        }
    }
}

#[cfg(test)]
mod test_var {
    use super::*;

    #[test]
    fn test_blank() {
        let result = Var::blank();
        let expected = Var {
            name: String::new(),
            format: String::new(),
            data: vec![],
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_new() {
        let result = Var::new("Name", "Format");
        let expected = Var {
            name: String::from("Name"),
            format: String::from("Format"),
            data: vec![],
        };
        assert_eq!(result, expected);
    }

    mod test_push {
        use super::*;

        #[test]
        fn empty() {
            let mut var = Var {
                name: String::new(),
                format: String::new(),
                data: vec![],
            };
            var.push(1.);
            assert_eq!(vec![1.], var.data);
        }

        #[test]
        fn double() {
            let mut var = Var {
                name: String::new(),
                format: String::new(),
                data: vec![],
            };
            var.push(1.);
            var.push(2.);
            assert_eq!(vec![1., 2.], var.data);
        }

        #[test]
        fn existing() {
            let mut var = Var {
                name: String::new(),
                format: String::new(),
                data: vec![1.],
            };
            var.push(2.);
            assert_eq!(vec![1., 2.], var.data);
        }
    }

    mod test_seq {
        use super::*;

        #[test]
        fn number_zero() {
            let mut var = Var {
                name: String::new(),
                format: String::new(),
                data: vec![],
            };
            var.seq(1., 2., 0);
            assert_eq!(Vec::<f64>::new(), var.data);
        }

        #[test]
        fn number_one() {
            let mut var = Var {
                name: String::new(),
                format: String::new(),
                data: vec![],
            };
            var.seq(10., 20., 1);
            assert_eq!(vec![10.], var.data);
        }

        #[test]
        fn simple() {
            let mut var = Var {
                name: String::new(),
                format: String::new(),
                data: vec![],
            };
            var.seq(1., 2., 2);
            assert_eq!(vec![1., 2.], var.data);
        }

        #[test]
        fn triple() {
            let mut var = Var {
                name: String::new(),
                format: String::new(),
                data: vec![],
            };
            var.seq(2000000000., 3000000000., 3);
            assert_eq!(vec![2000000000., 2500000000., 3000000000.], var.data);
        }

        #[test]
        fn reversed() {
            let mut var = Var {
                name: String::new(),
                format: String::new(),
                data: vec![],
            };
            var.seq(3000000000., 2000000000., 3);
            assert_eq!(vec![3000000000., 2500000000., 2000000000.], var.data);
        }
    }
}

/// Define a constant in the file
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

/// The file header
///
/// Note that the `DATA` keywords are not defined here.
#[derive(Debug, PartialEq, Clone)]
pub struct Header {
    pub version: String,
    pub name: String,
    pub comments: Vec<String>,
    pub devices: Vec<Device>,
    pub independent_variable: Var,
    pub constants: Vec<Constant>,
}

impl Default for Header {
    fn default() -> Self {
        Header {
            version: String::from("A.01.00"),
            name: String::new(),
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
            version: String::from(version),
            name: String::from(name),
            comments: vec![],
            devices: vec![],
            independent_variable: Var::blank(),
            constants: vec![],
        }
    }

    fn blank() -> Header {
        Header {
            version: String::new(),
            name: String::new(),
            comments: vec![],
            devices: vec![],
            independent_variable: Var::blank(),
            constants: vec![],
        }
    }

    pub fn add_device(&mut self, device_name: &str, value: &str) {
        self.create_device(device_name);
        if let Some(i) = self.index_device(device_name) {
            self.devices[i].entries.push(String::from(value));
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
            version: String::from("A.01.00"),
            name: String::new(),
            comments: vec![],
            devices: vec![],
            independent_variable: Var {
                name: String::new(),
                format: String::new(),
                data: vec![],
            },
            constants: vec![],
        };
        let result = Header::default();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_new() {
        let expected = Header {
            version: String::from("A.01.01"),
            name: String::from("A_NAME"),
            comments: vec![],
            devices: vec![],
            independent_variable: Var {
                name: String::new(),
                format: String::new(),
                data: vec![],
            },
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
                let expected = vec![Device {
                    name: String::from("NA"),
                    entries: vec![String::from("VERSION HP8510B.05.00")],
                }];
                let mut header = Header::new("A.01.01", "A_NAME");
                header.add_device("NA", "VERSION HP8510B.05.00");
                assert_eq!(header.devices, expected);
            }

            #[test]
            fn double_add() {
                let expected = vec![Device {
                    name: String::from("NA"),
                    entries: vec![
                        String::from("VERSION HP8510B.05.00"),
                        String::from("REGISTER 1"),
                    ],
                }];
                let mut header = Header::new("A.01.01", "A_NAME");
                header.add_device("NA", "VERSION HP8510B.05.00");
                header.add_device("NA", "REGISTER 1");
                assert_eq!(header.devices, expected);
            }

            #[test]
            fn add_two_devices() {
                let expected = vec![
                    Device {
                        name: String::from("NA"),
                        entries: vec![String::from("VERSION HP8510B.05.00")],
                    },
                    Device {
                        name: String::from("WVI"),
                        entries: vec![String::from("REGISTER 1")],
                    },
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
                let expected = vec![Device {
                    name: String::from("A Name"),
                    entries: vec![],
                }];
                let mut header = Header::new("A.01.01", "A_NAME");
                header.create_device("A Name");
                assert_eq!(header.devices, expected);
            }

            #[test]
            fn appends_device() {
                let expected = vec![
                    Device {
                        name: String::from("Different Name"),
                        entries: vec![],
                    },
                    Device {
                        name: String::from("A Name"),
                        entries: vec![],
                    },
                ];
                let mut header = Header::new("A.01.01", "A_NAME");
                header.create_device("Different Name");
                header.create_device("A Name");
                assert_eq!(header.devices, expected);
            }

            #[test]
            fn existing_device() {
                let expected = vec![Device {
                    name: String::from("A Name"),
                    entries: vec![],
                }];
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
                assert_eq!(
                    header.get_device_by_name("A Name"),
                    Some(&Device {
                        name: String::from("A Name"),
                        entries: vec![]
                    })
                );
            }
        }
    }
}

/// A named, formatted, data array
///
/// Consistency of the format with the variable `samples` is not
/// guaranteed and should be enforced by users of this code.
#[derive(Debug, PartialEq, Clone)]
pub struct DataArray {
    pub name: String,
    pub format: String,
    pub samples: Vec<Complex<f64>>,
}

impl DataArray {
    #[cfg(test)]
    fn blank() -> DataArray {
        DataArray {
            name: String::new(),
            format: String::new(),
            samples: vec![],
        }
    }

    pub fn new(name: &str, format: &str) -> DataArray {
        DataArray {
            name: String::from(name),
            format: String::from(format),
            samples: vec![],
        }
    }

    pub fn add_sample(&mut self, real: f64, imag: f64) {
        self.samples.push(Complex::<f64>::new(real, imag));
    }
}

#[cfg(test)]
mod test_data_array {
    use super::*;

    #[test]
    fn test_blank() {
        let expected = DataArray {
            name: String::new(),
            format: String::new(),
            samples: vec![],
        };
        let result = DataArray::blank();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_new() {
        let expected = DataArray {
            name: String::from("S"),
            format: String::from("RI"),
            samples: vec![],
        };
        let result = DataArray::new("S", "RI");
        assert_eq!(result, expected);
    }

    #[cfg(test)]
    mod test_add_sample {
        use super::*;

        #[test]
        fn empty() {
            let mut result = DataArray {
                name: String::new(),
                format: String::new(),
                samples: vec![],
            };
            result.add_sample(1., 2.);
            assert_complex_array_relative_eq!(result.samples, vec![Complex { re: 1., im: 2. }]);
        }

        #[test]
        fn double() {
            let mut result = DataArray {
                name: String::new(),
                format: String::new(),
                samples: vec![],
            };
            result.add_sample(1., 2.);
            result.add_sample(-1., -2.);
            assert_complex_array_relative_eq!(
                result.samples,
                vec![Complex { re: 1., im: 2. }, Complex { re: -1., im: -2. }]
            );
        }

        #[test]
        fn existing() {
            let mut result = DataArray {
                name: String::new(),
                format: String::new(),
                samples: vec![Complex { re: 1., im: 2. }],
            };
            result.add_sample(3., 4.);
            assert_complex_array_relative_eq!(
                result.samples,
                vec![Complex { re: 1., im: 2. }, Complex { re: 3., im: 4. }]
            );
        }
    }
}

/// Representation of a file
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

/// Error during writing
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
        fn writting_error() {
            let error = WriteError::WrittingError(std::io::ErrorKind::NotFound.into());
            assert_eq!(
                format!("{}", error),
                "Writing error occured: entity not found"
            );
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

    /// Read record
    ///
    /// Example usage:
    /// ```no_run
    /// use citi::Record;
    /// use std::fs::File;
    ///
    /// let mut file = File::open("file.cti").unwrap();
    /// let record = Record::from_reader(&mut file);
    /// ```
    pub fn from_reader<R: std::io::Read>(reader: &mut R) -> Result<Record> {
        let mut state = RecordReaderState::new();

        let buf_reader = std::io::BufReader::new(reader);
        for (i, line) in buf_reader.lines().enumerate() {
            let this_line = line.map_err(ReadError::ReadingError)?;
            // Filter out new lines
            if !this_line.trim().is_empty() {
                let keyword =
                    Keyword::from_str(&this_line).map_err(|e| ReadError::LineError(i, e))?;
                state = state.process_keyword(keyword)?;
            }
        }

        Ok(state.validate_record()?.record)
    }

    /// Write record
    ///
    /// Example usage:
    /// ```no_run
    /// use citi::Record;
    /// use std::fs::File;
    ///
    /// let record = Record::default();
    /// let mut file = File::create("file.cti").unwrap();
    /// record.to_writer(&mut file);
    /// ```
    pub fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        let keywords = self.get_keywords()?;

        for keyword in keywords.iter() {
            writeln!(writer, "{}", keyword).map_err(WriteError::WrittingError)?;
        }

        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn get_data_keywords(&self) -> WriteResult<Vec<Keyword>> {
        let mut keywords: Vec<Keyword> = vec![];

        // Add each array
        for array in self.data.iter() {
            keywords.push(Keyword::Begin);

            for Complex { re: real, im: imag } in array.samples.iter() {
                keywords.push(Keyword::DataPair {
                    real: *real,
                    imag: *imag,
                });
            }
            keywords.push(Keyword::End);
        }

        Ok(keywords)
    }

    fn get_data_defines_keywords(&self) -> WriteResult<Vec<Keyword>> {
        let mut keywords: Vec<Keyword> = vec![];

        for (i, array) in self.data.iter().enumerate() {
            match (array.name.is_empty(), array.format.is_empty()) {
                (true, _) => return Err(WriteError::NoDataName(i)),
                (_, true) => return Err(WriteError::NoDataFormat(i)),
                (_, _) => keywords.push(Keyword::Data {
                    name: array.name.clone(),
                    format: array.format.clone(),
                }),
            }
        }
        Ok(keywords)
    }

    fn get_version_keywords(&self) -> WriteResult<Vec<Keyword>> {
        match !self.header.version.is_empty() {
            true => Ok(vec![Keyword::CitiFile {
                version: self.header.version.clone(),
            }]),
            false => Err(WriteError::NoVersion),
        }
    }

    fn get_name_keywords(&self) -> WriteResult<Vec<Keyword>> {
        match !self.header.name.is_empty() {
            true => Ok(vec![Keyword::Name(self.header.name.clone())]),
            false => Err(WriteError::NoName),
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn get_comments_keywords(&self) -> WriteResult<Vec<Keyword>> {
        Ok(self
            .header
            .comments
            .iter()
            .map(|s| Keyword::Comment(s.clone()))
            .collect())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn get_devices_keywords(&self) -> WriteResult<Vec<Keyword>> {
        let mut keywords: Vec<Keyword> = vec![];

        for device in self.header.devices.iter() {
            for entry in device.entries.iter() {
                keywords.push(Keyword::Device {
                    name: device.name.clone(),
                    value: entry.clone(),
                });
            }
        }

        Ok(keywords)
    }

    #[allow(clippy::unnecessary_wraps)]
    fn get_independent_variable_keywords(&self) -> WriteResult<Vec<Keyword>> {
        Ok(vec![Keyword::Var {
            name: self.header.independent_variable.name.clone(),
            format: self.header.independent_variable.format.clone(),
            length: self.header.independent_variable.data.len(),
        }])
    }

    #[allow(clippy::unnecessary_wraps)]
    fn get_var_keywords(&self) -> WriteResult<Vec<Keyword>> {
        let mut keywords: Vec<Keyword> = vec![];

        // Do not set if length == 0
        if !self.header.independent_variable.data.is_empty() {
            keywords.push(Keyword::VarListBegin);
            for &v in self.header.independent_variable.data.iter() {
                keywords.push(Keyword::VarListItem(v));
            }
            keywords.push(Keyword::VarListEnd);
        }

        Ok(keywords)
    }

    #[allow(clippy::unnecessary_wraps)]
    fn get_constants_keywords(&self) -> WriteResult<Vec<Keyword>> {
        Ok(self
            .header
            .constants
            .iter()
            .map(|c| Keyword::Constant {
                name: c.name.clone(),
                value: c.value.clone(),
            })
            .collect())
    }

    fn get_keywords(&self) -> WriteResult<Vec<Keyword>> {
        let mut keywords: Vec<Keyword> = vec![];

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
}

#[cfg(test)]
mod test_record {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn write_gives_error_on_bad_file() {
        let record = Record::default();
        let mut file = std::fs::File::create(tempdir().unwrap().path().join("temp.cti")).unwrap();
        match record.to_writer(&mut file) {
            Err(Error::WriteError(WriteError::NoName)) => (),
            e => panic!("{:?}", e),
        }
    }

    mod test_write {
        use super::*;

        #[test]
        fn get_keywords() {
            let mut record = Record::default();
            record.header.constants.push(Constant {
                name: String::from("Const Name"),
                value: String::from("Value"),
            });
            record.header.independent_variable = Var {
                name: String::from("Var Name"),
                format: String::from("Format"),
                data: vec![1.],
            };
            record.header.devices.push(Device {
                name: String::from("Name A"),
                entries: vec![String::from("entry 1"), String::from("entry 2")],
            });
            record.header.comments.push(String::from("A Comment"));
            record.header.name = String::from("Name");
            record.header.version = String::from("A.01.00");
            record.data.push(DataArray {
                name: String::from("Data Name A"),
                format: String::from("Format A"),
                samples: vec![Complex { re: 1., im: 2. }],
            });
            record.data.push(DataArray {
                name: String::from("Data Name B"),
                format: String::from("Format B"),
                samples: vec![Complex { re: 3., im: 5. }, Complex { re: 4., im: 6. }],
            });

            match record.get_keywords() {
                Ok(v) => assert_eq!(
                    v,
                    vec![
                        Keyword::CitiFile {
                            version: String::from("A.01.00")
                        },
                        Keyword::Name(String::from("Name")),
                        Keyword::Var {
                            name: String::from("Var Name"),
                            format: String::from("Format"),
                            length: 1
                        },
                        Keyword::VarListBegin,
                        Keyword::VarListItem(1.),
                        Keyword::VarListEnd,
                        Keyword::Constant {
                            name: String::from("Const Name"),
                            value: String::from("Value")
                        },
                        Keyword::Comment(String::from("A Comment")),
                        Keyword::Device {
                            name: String::from("Name A"),
                            value: String::from("entry 1")
                        },
                        Keyword::Device {
                            name: String::from("Name A"),
                            value: String::from("entry 2")
                        },
                        Keyword::Data {
                            name: String::from("Data Name A"),
                            format: String::from("Format A")
                        },
                        Keyword::Data {
                            name: String::from("Data Name B"),
                            format: String::from("Format B")
                        },
                        Keyword::Begin,
                        Keyword::DataPair { real: 1., imag: 2. },
                        Keyword::End,
                        Keyword::Begin,
                        Keyword::DataPair { real: 3., imag: 5. },
                        Keyword::DataPair { real: 4., imag: 6. },
                        Keyword::End,
                    ]
                ),
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
                    Ok(v) => assert_eq!(
                        v,
                        vec![
                            Keyword::VarListBegin,
                            Keyword::VarListItem(1.),
                            Keyword::VarListEnd
                        ]
                    ),
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
                    Ok(v) => assert_eq!(
                        v,
                        vec![
                            Keyword::VarListBegin,
                            Keyword::VarListItem(1.),
                            Keyword::VarListItem(2.),
                            Keyword::VarListItem(3.),
                            Keyword::VarListEnd
                        ]
                    ),
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
                record.header.constants.push(Constant {
                    name: String::from("Name"),
                    value: String::from("Value"),
                });
                match record.get_constants_keywords() {
                    Ok(v) => assert_eq!(
                        v,
                        vec![Keyword::Constant {
                            name: String::from("Name"),
                            value: String::from("Value")
                        }]
                    ),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn two() {
                let mut record = Record::default();
                record.header.constants.push(Constant {
                    name: String::from("Name A"),
                    value: String::from("Value A"),
                });
                record.header.constants.push(Constant {
                    name: String::from("Name B"),
                    value: String::from("Value B"),
                });
                match record.get_constants_keywords() {
                    Ok(v) => assert_eq!(
                        v,
                        vec![
                            Keyword::Constant {
                                name: String::from("Name A"),
                                value: String::from("Value A")
                            },
                            Keyword::Constant {
                                name: String::from("Name B"),
                                value: String::from("Value B")
                            }
                        ]
                    ),
                    e => panic!("{:?}", e),
                }
            }
        }

        mod test_get_independent_variable_keywords {
            use super::*;

            #[test]
            fn no_format() {
                let mut record = Record::default();
                record.header.independent_variable.name = String::from("Name");
                match record.get_independent_variable_keywords() {
                    Ok(v) => assert_eq!(
                        v,
                        vec![Keyword::Var {
                            name: String::from("Name"),
                            format: String::new(),
                            length: 0
                        }]
                    ),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn format() {
                let mut record = Record::default();
                record.header.independent_variable.name = String::from("Name");
                record.header.independent_variable.format = String::from("Format");
                match record.get_independent_variable_keywords() {
                    Ok(v) => assert_eq!(
                        v,
                        vec![Keyword::Var {
                            name: String::from("Name"),
                            format: String::from("Format"),
                            length: 0
                        }]
                    ),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn length() {
                let mut record = Record::default();
                record.header.independent_variable.name = String::from("Name");
                record.header.independent_variable.format = String::from("Format");
                record.header.independent_variable.data = vec![0.; 10];
                match record.get_independent_variable_keywords() {
                    Ok(v) => assert_eq!(
                        v,
                        vec![Keyword::Var {
                            name: String::from("Name"),
                            format: String::from("Format"),
                            length: 10
                        }]
                    ),
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
                record.header.devices.push(Device {
                    name: String::from(""),
                    entries: vec![],
                });
                match record.get_devices_keywords() {
                    Ok(v) => assert_eq!(v, vec![]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn one_device() {
                let mut record = Record::default();
                record.header.devices.push(Device {
                    name: String::from("Name"),
                    entries: vec![String::from("entry")],
                });
                match record.get_devices_keywords() {
                    Ok(v) => assert_eq!(
                        v,
                        vec![Keyword::Device {
                            name: String::from("Name"),
                            value: String::from("entry")
                        }]
                    ),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn one_device_multiple_entries() {
                let mut record = Record::default();
                record.header.devices.push(Device {
                    name: String::from("Name"),
                    entries: vec![String::from("entry 1"), String::from("entry 2")],
                });
                match record.get_devices_keywords() {
                    Ok(v) => assert_eq!(
                        v,
                        vec![
                            Keyword::Device {
                                name: String::from("Name"),
                                value: String::from("entry 1")
                            },
                            Keyword::Device {
                                name: String::from("Name"),
                                value: String::from("entry 2")
                            }
                        ]
                    ),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn multiple_devices() {
                let mut record = Record::default();
                record.header.devices.push(Device {
                    name: String::from("Name A"),
                    entries: vec![String::from("entry 1")],
                });
                record.header.devices.push(Device {
                    name: String::from("Name B"),
                    entries: vec![String::from("entry 2")],
                });
                match record.get_devices_keywords() {
                    Ok(v) => assert_eq!(
                        v,
                        vec![
                            Keyword::Device {
                                name: String::from("Name A"),
                                value: String::from("entry 1")
                            },
                            Keyword::Device {
                                name: String::from("Name B"),
                                value: String::from("entry 2")
                            }
                        ]
                    ),
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
                    Ok(v) => assert_eq!(v, vec![Keyword::Comment(String::from("A Comment"))]),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn multiple() {
                let mut record = Record::default();
                record.header.comments.push(String::from("A Comment"));
                record.header.comments.push(String::from("B Comment"));
                match record.get_comments_keywords() {
                    Ok(v) => assert_eq!(
                        v,
                        vec![
                            Keyword::Comment(String::from("A Comment")),
                            Keyword::Comment(String::from("B Comment"))
                        ]
                    ),
                    e => panic!("{:?}", e),
                }
            }
        }

        mod test_get_name_keywords {
            use super::*;

            #[test]
            fn none() {
                let mut record = Record::default();
                record.header.name = String::new();
                match record.get_name_keywords() {
                    Err(WriteError::NoName) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn some() {
                let mut record = Record::default();
                record.header.name = String::from("A.01.00");
                match record.get_name_keywords() {
                    Ok(v) => assert_eq!(v, vec![Keyword::Name(String::from("A.01.00"))]),
                    e => panic!("{:?}", e),
                }
            }
        }

        mod test_get_version_keywords {
            use super::*;

            #[test]
            fn none() {
                let mut record = Record::default();
                record.header.version = String::new();
                match record.get_version_keywords() {
                    Err(WriteError::NoVersion) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn some() {
                let mut record = Record::default();
                record.header.version = String::from("A.01.00");
                match record.get_version_keywords() {
                    Ok(v) => assert_eq!(
                        v,
                        vec![Keyword::CitiFile {
                            version: String::from("A.01.00")
                        }]
                    ),
                    e => panic!("{:?}", e),
                }
            }
        }

        mod test_get_data_keywords {
            use super::*;

            #[test]
            fn many_values() {
                let mut record = Record::default();
                record.data.push(DataArray {
                    name: String::new(),
                    format: String::new(),
                    samples: vec![
                        Complex { re: 1., im: 4. },
                        Complex { re: 2., im: 1e-6 },
                        Complex { re: -3., im: 0. },
                    ],
                });
                match record.get_data_keywords() {
                    Ok(v) => assert_eq!(
                        v,
                        vec![
                            Keyword::Begin,
                            Keyword::DataPair { real: 1., imag: 4. },
                            Keyword::DataPair {
                                real: 2.,
                                imag: 1e-6
                            },
                            Keyword::DataPair {
                                real: -3.,
                                imag: 0.
                            },
                            Keyword::End
                        ]
                    ),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn one_array_gives_correct_result() {
                let mut record = Record::default();
                record.data.push(DataArray {
                    name: String::new(),
                    format: String::new(),
                    samples: vec![Complex { re: 1., im: 2. }],
                });
                match record.get_data_keywords() {
                    Ok(v) => assert_eq!(
                        v,
                        vec![
                            Keyword::Begin,
                            Keyword::DataPair { real: 1., imag: 2. },
                            Keyword::End
                        ]
                    ),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn multiple_array_gives_correct_result() {
                let mut record = Record::default();
                record.data.push(DataArray {
                    name: String::new(),
                    format: String::new(),
                    samples: vec![Complex { re: 1., im: 2. }],
                });
                record.data.push(DataArray {
                    name: String::new(),
                    format: String::new(),
                    samples: vec![Complex { re: 3., im: 4. }],
                });
                match record.get_data_keywords() {
                    Ok(v) => assert_eq!(
                        v,
                        vec![
                            Keyword::Begin,
                            Keyword::DataPair { real: 1., imag: 2. },
                            Keyword::End,
                            Keyword::Begin,
                            Keyword::DataPair { real: 3., imag: 4. },
                            Keyword::End
                        ]
                    ),
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
        }

        mod test_get_data_defines_keywords {
            use super::*;

            #[test]
            fn one_entry() {
                let mut record = Record::default();
                record.data.push(DataArray {
                    name: String::from("Name"),
                    format: String::from("Format"),
                    samples: vec![],
                });
                match record.get_data_defines_keywords() {
                    Ok(v) => assert_eq!(
                        v,
                        vec![Keyword::Data {
                            name: String::from("Name"),
                            format: String::from("Format")
                        }]
                    ),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn multiple() {
                let mut record = Record::default();
                record.data.push(DataArray {
                    name: String::from("Name A"),
                    format: String::from("Format A"),
                    samples: vec![],
                });
                record.data.push(DataArray {
                    name: String::from("Name B"),
                    format: String::from("Format B"),
                    samples: vec![],
                });
                match record.get_data_defines_keywords() {
                    Ok(v) => {
                        assert_eq!(
                            v,
                            vec![
                                Keyword::Data {
                                    name: String::from("Name A"),
                                    format: String::from("Format A")
                                },
                                Keyword::Data {
                                    name: String::from("Name B"),
                                    format: String::from("Format B")
                                }
                            ]
                        );
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn no_name() {
                let mut record = Record::default();
                record.data.push(DataArray {
                    name: String::new(),
                    format: String::from("Format"),
                    samples: vec![],
                });
                match record.get_data_defines_keywords() {
                    Err(WriteError::NoDataName(0)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn no_format() {
                let mut record = Record::default();
                record.data.push(DataArray {
                    name: String::from("Name"),
                    format: String::new(),
                    samples: vec![],
                });
                match record.get_data_defines_keywords() {
                    Err(WriteError::NoDataFormat(0)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn no_name_no_format() {
                let mut record = Record::default();
                record.data.push(DataArray {
                    name: String::new(),
                    format: String::new(),
                    samples: vec![],
                });
                match record.get_data_defines_keywords() {
                    Err(WriteError::NoDataName(0)) => (),
                    e => panic!("{:?}", e),
                }
            }
        }
    }

    #[cfg(test)]
    mod test_read {
        use super::*;

        #[test]
        fn cannot_read_empty_record() {
            match Record::from_reader(&mut "".as_bytes()) {
                Err(Error::ReadError(ReadError::NoName)) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn succeed_on_multiple_new_lines() {
            let contents = "CITIFILE A.01.00\nNAME MEMORY\n\n\n\n\n\n\n\n\nVAR FREQ MAG 3\nDATA S RI\nBEGIN\n-3.54545E-2,-1.38601E-3\n0.23491E-3,-1.39883E-3\n2.00382E-3,-1.40022E-3\nEND\n";
            match Record::from_reader(&mut contents.as_bytes()) {
                Ok(_) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn succeed_on_whitespace_new_lines() {
            let contents = "CITIFILE A.01.00\nNAME MEMORY\n      \n\n\n\n\n\n\n\nVAR FREQ MAG 3\nDATA S RI\nBEGIN\n-3.54545E-2,-1.38601E-3\n0.23491E-3,-1.39883E-3\n2.00382E-3,-1.40022E-3\nEND\n";
            match Record::from_reader(&mut contents.as_bytes()) {
                Ok(_) => (),
                e => panic!("{:?}", e),
            }
        }

        #[cfg(test)]
        mod test_read_minimal_record {
            use super::*;

            fn setup() -> Result<Record> {
                let contents = "CITIFILE A.01.00\nNAME MEMORY\nVAR FREQ MAG 3\nDATA S RI\nBEGIN\n-3.54545E-2,-1.38601E-3\n0.23491E-3,-1.39883E-3\n2.00382E-3,-1.40022E-3\nEND\n";
                Record::from_reader(&mut contents.as_bytes())
            }

            #[test]
            fn name() {
                match setup() {
                    Ok(record) => assert_eq!(record.header.name, "MEMORY"),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn version() {
                match setup() {
                    Ok(record) => assert_eq!(record.header.version, "A.01.00"),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn comments() {
                match setup() {
                    Ok(record) => assert_eq!(record.header.comments.len(), 0),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn constants() {
                match setup() {
                    Ok(record) => assert_eq!(record.header.constants.len(), 0),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn devices() {
                match setup() {
                    Ok(record) => assert_eq!(record.header.devices.len(), 0),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn independent_variable() {
                match setup() {
                    Ok(record) => {
                        assert_eq!(record.header.independent_variable.name, "FREQ");
                        assert_eq!(record.header.independent_variable.format, "MAG");
                        assert_eq!(record.header.independent_variable.data.len(), 0);
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn data() {
                match setup() {
                    Ok(record) => {
                        assert_eq!(record.data.len(), 1);
                        assert_eq!(record.data[0].name, "S");
                        assert_eq!(record.data[0].format, "RI");
                        assert_eq!(record.data[0].samples.len(), 3);
                        assert_complex_array_relative_eq!(
                            record.data[0].samples,
                            vec![
                                Complex {
                                    re: -0.0354545,
                                    im: -0.00138601
                                },
                                Complex {
                                    re: 0.00023491,
                                    im: -0.00139883
                                },
                                Complex {
                                    re: 0.00200382,
                                    im: -0.00140022
                                },
                            ]
                        );
                    }
                    e => panic!("{:?}", e),
                }
            }
        }
    }

    #[test]
    fn test_default() {
        let expected = Record {
            header: Header {
                version: String::from("A.01.00"),
                name: String::new(),
                comments: vec![],
                devices: vec![],
                independent_variable: Var {
                    name: String::new(),
                    format: String::new(),
                    data: vec![],
                },
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
                version: String::from("A.01.01"),
                name: String::from("A_NAME"),
                comments: vec![],
                devices: vec![],
                independent_variable: Var {
                    name: String::new(),
                    format: String::new(),
                    data: vec![],
                },
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
                version: String::new(),
                name: String::new(),
                comments: vec![],
                devices: vec![],
                independent_variable: Var {
                    name: String::new(),
                    format: String::new(),
                    data: vec![],
                },
                constants: vec![],
            },
            data: vec![],
        };
        let result = Record::blank();
        assert_eq!(result, expected);
    }
}

/// Error during reading
#[derive(Error, Debug)]
pub enum ReadError {
    #[error("More data arrays than defined in header")]
    DataArrayOverIndex,
    #[error("Independent variable defined twice")]
    IndependentVariableDefinedTwice,
    #[error("Single use keyword `{0}` defined twice")]
    SingleUseKeywordDefinedTwice(Keyword),
    #[error("Keyword `{0}` is out of order in the record")]
    OutOfOrderKeyword(Keyword),
    #[error("Error on line {0}: {1}")]
    LineError(usize, ParseError),
    #[error("Reading error occured: {0}")]
    ReadingError(std::io::Error),
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
}
type ReaderResult<T> = std::result::Result<T, ReadError>;

#[cfg(test)]
mod test_reader_error {
    use super::*;

    mod test_display {
        use super::*;

        #[test]
        fn data_array_over_index() {
            let error = ReadError::DataArrayOverIndex;
            assert_eq!(
                format!("{}", error),
                "More data arrays than defined in header"
            );
        }

        #[test]
        fn independent_variable_defined_twice() {
            let error = ReadError::IndependentVariableDefinedTwice;
            assert_eq!(format!("{}", error), "Independent variable defined twice");
        }

        #[test]
        fn single_use_keyword_defined_twice() {
            let error = ReadError::SingleUseKeywordDefinedTwice(Keyword::End);
            assert_eq!(
                format!("{}", error),
                "Single use keyword `END` defined twice"
            );
        }

        #[test]
        fn out_of_order_keyword() {
            let error = ReadError::OutOfOrderKeyword(Keyword::Begin);
            assert_eq!(
                format!("{}", error),
                "Keyword `BEGIN` is out of order in the record"
            );
        }

        #[test]
        fn reading_error() {
            let error = ReadError::ReadingError(std::io::ErrorKind::NotFound.into());
            assert_eq!(
                format!("{}", error),
                "Reading error occured: entity not found"
            );
        }

        #[test]
        fn line_error() {
            let error = ReadError::LineError(10, ParseError::BadRegex);
            assert_eq!(
                format!("{}", error),
                "Error on line 10: Regex could not be parsed"
            );
        }

        #[test]
        fn no_version() {
            let error = ReadError::NoVersion;
            assert_eq!(format!("{}", error), "Version is not defined");
        }

        #[test]
        fn no_name() {
            let error = ReadError::NoName;
            assert_eq!(format!("{}", error), "Name is not defined");
        }

        #[test]
        fn no_independent_variable() {
            let error = ReadError::NoIndependentVariable;
            assert_eq!(format!("{}", error), "Indepent variable is not defined");
        }

        #[test]
        fn no_data() {
            let error = ReadError::NoData;
            assert_eq!(format!("{}", error), "Data name and format is not defined");
        }

        #[test]
        fn var_and_data() {
            let error = ReadError::VarAndDataDifferentLengths(1, 2, 3);
            assert_eq!(
                format!("{}", error),
                "Independent variable and data array 3 are different lengths (1 != 2)"
            );
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
#[derive(Debug, PartialEq, Clone)]
struct RecordReaderState {
    record: Record,
    state: RecordReaderStates,
    data_array_counter: usize,
    independent_variable_already_read: bool,
    version_aready_read: bool,
    name_already_read: bool,
    var_already_read: bool,
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
            version_aready_read: false,
            name_already_read: false,
            var_already_read: false,
        }
    }

    pub fn process_keyword(self, keyword: Keyword) -> ReaderResult<Self> {
        match self.state {
            RecordReaderStates::Header => RecordReaderState::state_header(self, keyword),
            RecordReaderStates::Data => RecordReaderState::state_data(self, keyword),
            RecordReaderStates::VarList => RecordReaderState::state_var_list(self, keyword),
            RecordReaderStates::SeqList => RecordReaderState::state_seq_list(self, keyword),
        }
    }

    fn state_header(mut self, keyword: Keyword) -> ReaderResult<Self> {
        match keyword {
            Keyword::CitiFile { version } => match self.version_aready_read {
                true => Err(ReadError::SingleUseKeywordDefinedTwice(Keyword::CitiFile {
                    version,
                })),
                false => {
                    self.version_aready_read = true;
                    self.record.header.version = version;
                    Ok(self)
                }
            },
            Keyword::Name(name) => match self.name_already_read {
                true => Err(ReadError::SingleUseKeywordDefinedTwice(Keyword::Name(name))),
                false => {
                    self.name_already_read = true;
                    self.record.header.name = name;
                    Ok(self)
                }
            },
            Keyword::Device { name, value } => {
                self.record.header.add_device(&name, &value);
                Ok(self)
            }
            Keyword::Comment(comment) => {
                self.record.header.comments.push(comment);
                Ok(self)
            }
            Keyword::Constant { name, value } => {
                self.record
                    .header
                    .constants
                    .push(Constant::new(&name, &value));
                Ok(self)
            }
            Keyword::Var {
                name,
                format,
                length,
            } => match self.var_already_read {
                true => Err(ReadError::SingleUseKeywordDefinedTwice(Keyword::Var {
                    name,
                    format,
                    length,
                })),
                false => {
                    self.var_already_read = true;
                    self.record.header.independent_variable.name = name;
                    self.record.header.independent_variable.format = format;
                    Ok(self)
                }
            },
            Keyword::VarListBegin => match self.independent_variable_already_read {
                false => {
                    self.state = RecordReaderStates::VarList;
                    Ok(self)
                }
                true => Err(ReadError::IndependentVariableDefinedTwice),
            },
            Keyword::SegListBegin => match self.independent_variable_already_read {
                false => {
                    self.state = RecordReaderStates::SeqList;
                    Ok(self)
                }
                true => Err(ReadError::IndependentVariableDefinedTwice),
            },
            Keyword::Begin => {
                self.state = RecordReaderStates::Data;
                Ok(self)
            }
            Keyword::Data { name, format } => {
                self.record.data.push(DataArray::new(&name, &format));
                Ok(self)
            }
            _ => Err(ReadError::OutOfOrderKeyword(keyword)),
        }
    }

    fn state_data(mut self, keyword: Keyword) -> ReaderResult<Self> {
        match keyword {
            Keyword::DataPair { real, imag } => {
                if self.data_array_counter < self.record.data.len() {
                    self.record.data[self.data_array_counter].add_sample(real, imag);
                    Ok(self)
                } else {
                    Err(ReadError::DataArrayOverIndex)
                }
            }
            Keyword::End => {
                self.state = RecordReaderStates::Header;
                self.data_array_counter += 1;
                Ok(self)
            }
            _ => Err(ReadError::OutOfOrderKeyword(keyword)),
        }
    }

    fn state_var_list(mut self, keyword: Keyword) -> ReaderResult<Self> {
        match keyword {
            Keyword::VarListItem(value) => {
                self.record.header.independent_variable.push(value);
                Ok(self)
            }
            Keyword::VarListEnd => {
                self.independent_variable_already_read = true;
                self.state = RecordReaderStates::Header;
                Ok(self)
            }
            _ => Err(ReadError::OutOfOrderKeyword(keyword)),
        }
    }

    fn state_seq_list(mut self, keyword: Keyword) -> ReaderResult<Self> {
        match keyword {
            Keyword::SegItem {
                first,
                last,
                number,
            } => {
                self.record
                    .header
                    .independent_variable
                    .seq(first, last, number);
                Ok(self)
            }
            Keyword::SegListEnd => {
                self.independent_variable_already_read = true;
                self.state = RecordReaderStates::Header;
                Ok(self)
            }
            _ => Err(ReadError::OutOfOrderKeyword(keyword)),
        }
    }

    pub fn validate_record(self) -> ReaderResult<Self> {
        self.has_name()?
            .has_version()?
            .has_var()?
            .has_data()?
            .var_and_data_same_length()
    }

    fn has_version(self) -> ReaderResult<Self> {
        match self.version_aready_read {
            true => Ok(self),
            false => Err(ReadError::NoVersion),
        }
    }

    fn has_name(self) -> ReaderResult<Self> {
        match self.name_already_read {
            true => Ok(self),
            false => Err(ReadError::NoName),
        }
    }

    fn has_var(self) -> ReaderResult<Self> {
        match self.var_already_read {
            true => Ok(self),
            false => Err(ReadError::NoIndependentVariable),
        }
    }

    fn has_data(self) -> ReaderResult<Self> {
        match self.record.data.len() {
            0 => Err(ReadError::NoData),
            _ => Ok(self),
        }
    }

    /// Zero length var with variable length data allowed
    fn var_and_data_same_length(self) -> ReaderResult<Self> {
        let mut n = self.record.header.independent_variable.data.len();

        for (i, data_array) in self.record.data.iter().enumerate() {
            let k = data_array.samples.len();
            if n == 0 {
                n = k
            } else if n != k {
                return Err(ReadError::VarAndDataDifferentLengths(n, k, i));
            }
        }
        Ok(self)
    }
}

#[cfg(test)]
mod test_record_reader_state {
    use super::*;
    use approx::*;

    #[test]
    fn test_new() {
        let expected = RecordReaderState {
            record: Record {
                header: Header {
                    version: String::new(),
                    name: String::new(),
                    comments: vec![],
                    devices: vec![],
                    independent_variable: Var {
                        name: String::new(),
                        format: String::new(),
                        data: vec![],
                    },
                    constants: vec![],
                },
                data: vec![],
            },
            state: RecordReaderStates::Header,
            data_array_counter: 0,
            independent_variable_already_read: false,
            version_aready_read: false,
            name_already_read: false,
            var_already_read: false,
        };
        let result = RecordReaderState::new();
        assert_eq!(result, expected);
    }

    mod test_state_header {
        use super::*;

        mod test_keywords {
            use super::*;

            fn initialize_state() -> RecordReaderState {
                RecordReaderState {
                    state: RecordReaderStates::Header,
                    ..RecordReaderState::new()
                }
            }

            #[test]
            fn citirecord() {
                let keyword = Keyword::CitiFile {
                    version: String::from("A.01.01"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.version, "A.01.01");
                        assert_eq!(s.state, RecordReaderStates::Header);
                        assert_eq!(s.version_aready_read, true);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn citirecord_cannot_be_called_twice() {
                let keyword = Keyword::CitiFile {
                    version: String::from("A.01.01"),
                };
                let mut state = initialize_state();
                state.version_aready_read = true;
                match state.process_keyword(keyword) {
                    Err(ReadError::SingleUseKeywordDefinedTwice(Keyword::CitiFile { version })) => {
                        assert_eq!(version, "A.01.01")
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn name() {
                let keyword = Keyword::Name(String::from("Name"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.name, "Name");
                        assert_eq!(s.state, RecordReaderStates::Header);
                        assert_eq!(s.name_already_read, true);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn name_cannot_be_called_twice() {
                let keyword = Keyword::Name(String::from("CAL_SET"));
                let mut state = initialize_state();
                state.name_already_read = true;
                match state.process_keyword(keyword) {
                    Err(ReadError::SingleUseKeywordDefinedTwice(Keyword::Name(name))) => {
                        assert_eq!(name, "CAL_SET")
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var() {
                let keyword = Keyword::Var {
                    name: String::from("Name"),
                    format: String::from("MAG"),
                    length: 102,
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.independent_variable.name, "Name");
                        assert_eq!(s.record.header.independent_variable.format, "MAG");
                        assert_eq!(s.state, RecordReaderStates::Header);
                        assert_eq!(s.var_already_read, true);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_cannot_be_called_twice() {
                let keyword = Keyword::Var {
                    name: String::from("FREQ"),
                    format: String::from("MAG"),
                    length: 102,
                };
                let mut state = initialize_state();
                state.var_already_read = true;
                match state.process_keyword(keyword) {
                    Err(ReadError::SingleUseKeywordDefinedTwice(Keyword::Var {
                        name,
                        format,
                        length,
                    })) => {
                        assert_eq!(name, "FREQ");
                        assert_eq!(format, "MAG");
                        assert_eq!(length, 102);
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn constant_empty() {
                let keyword = Keyword::Constant {
                    name: String::from("Name"),
                    value: String::from("Value"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(
                            s.record.header.constants,
                            vec![Constant::new("Name", "Value")]
                        );
                        assert_eq!(s.state, RecordReaderStates::Header);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn constant_exists() {
                let keyword = Keyword::Constant {
                    name: String::from("New Name"),
                    value: String::from("New Value"),
                };
                let mut state = initialize_state();
                state
                    .record
                    .header
                    .constants
                    .push(Constant::new("Name", "Value"));
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(
                            s.record.header.constants,
                            vec![
                                Constant::new("Name", "Value"),
                                Constant::new("New Name", "New Value")
                            ]
                        );
                        assert_eq!(s.state, RecordReaderStates::Header);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn device() {
                let keyword = Keyword::Device {
                    name: String::from("NA"),
                    value: String::from("Value"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.devices.len(), 1);
                        assert_eq!(
                            s.record.header.devices[0],
                            Device {
                                name: String::from("NA"),
                                entries: vec![String::from("Value")]
                            }
                        );
                        assert_eq!(s.state, RecordReaderStates::Header);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn device_with_existing_device() {
                let keyword = Keyword::Device {
                    name: String::from("WVI"),
                    value: String::from("1904"),
                };
                let mut state = initialize_state();
                state.record.header.add_device("NA", "Value");
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.devices.len(), 2);
                        assert_eq!(
                            s.record.header.devices[0],
                            Device {
                                name: String::from("NA"),
                                entries: vec![String::from("Value")]
                            }
                        );
                        assert_eq!(
                            s.record.header.devices[1],
                            Device {
                                name: String::from("WVI"),
                                entries: vec![String::from("1904")]
                            }
                        );
                        assert_eq!(s.state, RecordReaderStates::Header);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_begin() {
                let keyword = Keyword::SegListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => assert_eq!(s.state, RecordReaderStates::SeqList),
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_begin_when_already_read() {
                let keyword = Keyword::SegListBegin;
                let mut state = initialize_state();
                state.independent_variable_already_read = true;
                match state.process_keyword(keyword) {
                    Err(ReadError::IndependentVariableDefinedTwice) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_item() {
                let keyword = Keyword::SegItem {
                    first: 10.,
                    last: 100.,
                    number: 2,
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::SegItem {
                        first,
                        last,
                        number,
                    })) => {
                        assert_relative_eq!(first, 10.);
                        assert_relative_eq!(last, 100.);
                        assert_eq!(number, 2);
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_end() {
                let keyword = Keyword::SegListEnd;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::SegListEnd)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_begin() {
                let keyword = Keyword::VarListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => assert_eq!(s.state, RecordReaderStates::VarList),
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_begin_when_already_read() {
                let keyword = Keyword::VarListBegin;
                let mut state = initialize_state();
                state.independent_variable_already_read = true;
                match state.process_keyword(keyword) {
                    Err(ReadError::IndependentVariableDefinedTwice) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_item() {
                let keyword = Keyword::VarListItem(1.);
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::VarListItem(f))) => {
                        assert_relative_eq!(f, 1.)
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn data() {
                let keyword = Keyword::Data {
                    name: String::from("S[1,1]"),
                    format: String::from("RI"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(
                            s.record.data,
                            vec![DataArray {
                                name: String::from("S[1,1]"),
                                format: String::from("RI"),
                                samples: vec![]
                            }]
                        );
                        assert_eq!(s.state, RecordReaderStates::Header);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn data_with_already_existing() {
                let keyword = Keyword::Data {
                    name: String::from("S[1,1]"),
                    format: String::from("RI"),
                };
                let mut state = initialize_state();
                state.record.data.push(DataArray {
                    name: String::from("E"),
                    format: String::from("RI"),
                    samples: vec![],
                });
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(
                            s.record.data,
                            vec![
                                DataArray {
                                    name: String::from("E"),
                                    format: String::from("RI"),
                                    samples: vec![]
                                },
                                DataArray {
                                    name: String::from("S[1,1]"),
                                    format: String::from("RI"),
                                    samples: vec![]
                                }
                            ]
                        );
                        assert_eq!(s.state, RecordReaderStates::Header);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn data_pair() {
                let keyword = Keyword::DataPair { real: 1., imag: 2. };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::DataPair { real, imag })) => {
                        assert_relative_eq!(real, 1.);
                        assert_relative_eq!(imag, 2.);
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn begin() {
                let keyword = Keyword::Begin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.data_array_counter, 0);
                        assert_eq!(s.state, RecordReaderStates::Data);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn end() {
                let keyword = Keyword::End;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::End)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn comment() {
                let keyword = Keyword::Comment(String::from("Comment"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.comments, vec![String::from("Comment")]);
                        assert_eq!(s.state, RecordReaderStates::Header);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn comment_with_existing() {
                let keyword = Keyword::Comment(String::from("Comment"));
                let mut state = initialize_state();
                state
                    .record
                    .header
                    .comments
                    .push(String::from("Comment First"));
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(
                            s.record.header.comments,
                            vec![String::from("Comment First"), String::from("Comment")]
                        );
                        assert_eq!(s.state, RecordReaderStates::Header);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }
        }
    }

    mod test_state_data {
        use super::*;

        mod test_keywords {
            use super::*;

            fn initialize_state() -> RecordReaderState {
                let mut state = RecordReaderState {
                    state: RecordReaderStates::Data,
                    ..RecordReaderState::new()
                };
                state.record.data.push(DataArray::blank());
                state
            }

            #[test]
            fn citirecord() {
                let keyword = Keyword::CitiFile {
                    version: String::from("A.01.01"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::CitiFile { version })) => {
                        assert_eq!(version, "A.01.01")
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn name() {
                let keyword = Keyword::Name(String::from("Name"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Name(name))) => {
                        assert_eq!(name, "Name")
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var() {
                let keyword = Keyword::Var {
                    name: String::from("Name"),
                    format: String::from("MAG"),
                    length: 102,
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Var {
                        name,
                        format,
                        length,
                    })) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, "MAG");
                        assert_eq!(length, 102);
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn constant() {
                let keyword = Keyword::Constant {
                    name: String::from("Name"),
                    value: String::from("Value"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Constant { name, value })) => {
                        assert_eq!(name, "Name");
                        assert_eq!(value, "Value");
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn device() {
                let keyword = Keyword::Device {
                    name: String::from("Name"),
                    value: String::from("Value"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Device { name, value })) => {
                        assert_eq!(name, "Name");
                        assert_eq!(value, "Value");
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_begin() {
                let keyword = Keyword::SegListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::SegListBegin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_item() {
                let keyword = Keyword::SegItem {
                    first: 10.,
                    last: 100.,
                    number: 2,
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::SegItem {
                        first,
                        last,
                        number,
                    })) => {
                        assert_relative_eq!(first, 10.);
                        assert_relative_eq!(last, 100.);
                        assert_eq!(number, 2);
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_end() {
                let keyword = Keyword::SegListEnd;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::SegListEnd)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_begin() {
                let keyword = Keyword::VarListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::VarListBegin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_item() {
                let keyword = Keyword::VarListItem(1.);
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::VarListItem(f))) => {
                        assert_relative_eq!(f, 1.)
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_item_exponent() {
                let keyword = Keyword::VarListItem(1e9);
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::VarListItem(f))) => {
                        assert_relative_eq!(f, 1e9)
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_end() {
                let keyword = Keyword::VarListEnd;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::VarListEnd)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn data() {
                let keyword = Keyword::Data {
                    name: String::from("Name"),
                    format: String::from("Format"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Data { name, format })) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, "Format");
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn data_pair() {
                let keyword = Keyword::DataPair { real: 1., imag: 2. };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.data.len(), 1);
                        assert_complex_array_relative_eq!(
                            s.record.data[0].samples,
                            vec![Complex { re: 1., im: 2. }]
                        );
                        assert_eq!(s.state, RecordReaderStates::Data);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn data_pair_second_array() {
                let keyword = Keyword::DataPair { real: 1., imag: 2. };
                let mut state = initialize_state();
                state.record.data.push(DataArray::blank());
                state.data_array_counter = 1;
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.data.len(), 2);
                        assert_eq!(s.record.data[0].samples, vec![]);
                        assert_complex_array_relative_eq!(
                            s.record.data[1].samples,
                            vec![Complex { re: 1., im: 2. }]
                        );
                        assert_eq!(s.state, RecordReaderStates::Data);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn data_pair_out_of_bounds() {
                let keyword = Keyword::DataPair { real: 1., imag: 2. };
                let mut state = initialize_state();
                state.data_array_counter = 1;
                match state.process_keyword(keyword) {
                    Err(ReadError::DataArrayOverIndex) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn begin() {
                let keyword = Keyword::Begin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Begin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn end() {
                let keyword = Keyword::End;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.state, RecordReaderStates::Header);
                        assert_eq!(s.data_array_counter, 1);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn end_increment_index() {
                let keyword = Keyword::End;
                let mut state = initialize_state();
                state.data_array_counter = 1;
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.state, RecordReaderStates::Header);
                        assert_eq!(s.data_array_counter, 2);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn comment() {
                let keyword = Keyword::Comment(String::from("Comment"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Comment(comment))) => {
                        assert_eq!(comment, "Comment")
                    }
                    e => panic!("{:?}", e),
                }
            }
        }
    }

    mod test_state_var_list {
        use super::*;

        mod test_keywords {
            use super::*;

            fn initialize_state() -> RecordReaderState {
                RecordReaderState {
                    state: RecordReaderStates::VarList,
                    ..RecordReaderState::new()
                }
            }

            #[test]
            fn citirecord() {
                let keyword = Keyword::CitiFile {
                    version: String::from("A.01.01"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::CitiFile { version })) => {
                        assert_eq!(version, "A.01.01")
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn name() {
                let keyword = Keyword::Name(String::from("Name"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Name(name))) => {
                        assert_eq!(name, "Name")
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var() {
                let keyword = Keyword::Var {
                    name: String::from("Name"),
                    format: String::from("MAG"),
                    length: 102,
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Var {
                        name,
                        format,
                        length,
                    })) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, "MAG");
                        assert_eq!(length, 102);
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn constant() {
                let keyword = Keyword::Constant {
                    name: String::from("Name"),
                    value: String::from("Value"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Constant { name, value })) => {
                        assert_eq!(name, "Name");
                        assert_eq!(value, "Value");
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn device() {
                let keyword = Keyword::Device {
                    name: String::from("Name"),
                    value: String::from("Value"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Device { name, value })) => {
                        assert_eq!(name, "Name");
                        assert_eq!(value, "Value");
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_begin() {
                let keyword = Keyword::SegListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::SegListBegin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_item() {
                let keyword = Keyword::SegItem {
                    first: 10.,
                    last: 100.,
                    number: 2,
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::SegItem {
                        first,
                        last,
                        number,
                    })) => {
                        assert_relative_eq!(first, 10.);
                        assert_relative_eq!(last, 100.);
                        assert_eq!(number, 2);
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_end() {
                let keyword = Keyword::SegListEnd;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::SegListEnd)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_begin() {
                let keyword = Keyword::VarListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::VarListBegin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_item() {
                let keyword = Keyword::VarListItem(1.);
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.independent_variable.data, vec![1.]);
                        assert_eq!(s.state, RecordReaderStates::VarList);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_item_exponent() {
                let keyword = Keyword::VarListItem(1e9);
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.independent_variable.data, vec![1e9]);
                        assert_eq!(s.state, RecordReaderStates::VarList);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_item_already_exists() {
                let keyword = Keyword::VarListItem(1e9);
                let mut state = initialize_state();
                state.record.header.independent_variable.push(1e8);
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.independent_variable.data, vec![1e8, 1e9]);
                        assert_eq!(s.state, RecordReaderStates::VarList);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_end() {
                let keyword = Keyword::VarListEnd;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.independent_variable_already_read, true);
                        assert_eq!(s.state, RecordReaderStates::Header);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn data() {
                let keyword = Keyword::Data {
                    name: String::from("Name"),
                    format: String::from("Format"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Data { name, format })) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, "Format");
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn data_pair() {
                let keyword = Keyword::DataPair { real: 1., imag: 1. };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::DataPair { real, imag })) => {
                        assert_relative_eq!(real, 1.);
                        assert_relative_eq!(imag, 1.);
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn begin() {
                let keyword = Keyword::Begin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Begin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn end() {
                let keyword = Keyword::End;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::End)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn comment() {
                let keyword = Keyword::Comment(String::from("Comment"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Comment(comment))) => {
                        assert_eq!(comment, "Comment")
                    }
                    e => panic!("{:?}", e),
                }
            }
        }
    }

    mod test_state_seq_list {
        use super::*;

        mod test_keywords {
            use super::*;

            fn initialize_state() -> RecordReaderState {
                RecordReaderState {
                    state: RecordReaderStates::SeqList,
                    ..RecordReaderState::new()
                }
            }

            #[test]
            fn citirecord() {
                let keyword = Keyword::CitiFile {
                    version: String::from("A.01.01"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::CitiFile { version })) => {
                        assert_eq!(version, "A.01.01")
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn name() {
                let keyword = Keyword::Name(String::from("Name"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Name(name))) => {
                        assert_eq!(name, "Name")
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var() {
                let keyword = Keyword::Var {
                    name: String::from("Name"),
                    format: String::from("MAG"),
                    length: 102,
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Var {
                        name,
                        format,
                        length,
                    })) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, "MAG");
                        assert_eq!(length, 102);
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn constant() {
                let keyword = Keyword::Constant {
                    name: String::from("Name"),
                    value: String::from("Value"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Constant { name, value })) => {
                        assert_eq!(name, "Name");
                        assert_eq!(value, "Value");
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn device() {
                let keyword = Keyword::Device {
                    name: String::from("Name"),
                    value: String::from("Value"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Device { name, value })) => {
                        assert_eq!(name, "Name");
                        assert_eq!(value, "Value");
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_begin() {
                let keyword = Keyword::SegListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::SegListBegin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_item() {
                let keyword = Keyword::SegItem {
                    first: 10.,
                    last: 100.,
                    number: 2,
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.record.header.independent_variable.data, vec![10., 100.]);
                        assert_eq!(s.state, RecordReaderStates::SeqList);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_item_triple() {
                let keyword = Keyword::SegItem {
                    first: 10.,
                    last: 100.,
                    number: 3,
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(
                            s.record.header.independent_variable.data,
                            vec![10., 55., 100.]
                        );
                        assert_eq!(s.state, RecordReaderStates::SeqList);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn seg_list_end() {
                let keyword = Keyword::SegListEnd;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Ok(s) => {
                        assert_eq!(s.independent_variable_already_read, true);
                        assert_eq!(s.state, RecordReaderStates::Header);
                    }
                    Err(e) => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_begin() {
                let keyword = Keyword::VarListBegin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::VarListBegin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_item() {
                let keyword = Keyword::VarListItem(1.);
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::VarListItem(f))) => {
                        assert_relative_eq!(f, 1.0)
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn var_list_end() {
                let keyword = Keyword::VarListEnd;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::VarListEnd)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn data() {
                let keyword = Keyword::Data {
                    name: String::from("Name"),
                    format: String::from("Format"),
                };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Data { name, format })) => {
                        assert_eq!(name, "Name");
                        assert_eq!(format, "Format");
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn data_pair() {
                let keyword = Keyword::DataPair { real: 1., imag: 1. };
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::DataPair { real, imag })) => {
                        assert_relative_eq!(real, 1.);
                        assert_relative_eq!(imag, 1.);
                    }
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn begin() {
                let keyword = Keyword::Begin;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Begin)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn end() {
                let keyword = Keyword::End;
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::End)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn comment() {
                let keyword = Keyword::Comment(String::from("Comment"));
                let state = initialize_state();
                match state.process_keyword(keyword) {
                    Err(ReadError::OutOfOrderKeyword(Keyword::Comment(s))) => {
                        assert_eq!(s, "Comment")
                    }
                    e => panic!("{:?}", e),
                }
            }
        }
    }

    #[cfg(test)]
    mod test_validate_record {
        use super::*;

        fn create_valid_record() -> Record {
            let mut record = Record::blank();
            record.header.name = String::from("CAL_SET");
            record.header.version = String::from("A.01.00");
            record.data.push(DataArray::new("E", "RI"));
            record.header.independent_variable.name = String::from("FREQ");
            record
        }

        fn create_valid_state() -> RecordReaderState {
            let mut state = RecordReaderState::new();
            state.record = create_valid_record();
            state.independent_variable_already_read = true;
            state.version_aready_read = true;
            state.name_already_read = true;
            state.var_already_read = true;
            state
        }

        #[test]
        fn test_valid_record() {
            let state = create_valid_state();
            match state.validate_record() {
                Ok(s) => assert_eq!(s.record, create_valid_record()),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn test_no_data() {
            let mut state = create_valid_state();
            state.record.data = vec![];
            match state.validate_record() {
                Err(ReadError::NoData) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn test_no_version() {
            let mut state = create_valid_state();
            state.version_aready_read = false;
            match state.validate_record() {
                Err(ReadError::NoVersion) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn test_no_name() {
            let mut state = create_valid_state();
            state.name_already_read = false;
            match state.validate_record() {
                Err(ReadError::NoName) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn test_no_var() {
            let mut state = create_valid_state();
            state.var_already_read = false;
            match state.validate_record() {
                Err(ReadError::NoIndependentVariable) => (),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn test_var_and_data_different() {
            let mut state = create_valid_state();
            state.record.data.push(DataArray {
                name: String::new(),
                format: String::new(),
                samples: vec![Complex { re: 1., im: 1. }, Complex { re: 1., im: 1. }],
            });
            state.record.header.independent_variable.data = vec![1.];
            match state.validate_record() {
                Err(ReadError::VarAndDataDifferentLengths(1, 0, 0)) => (),
                e => panic!("{:?}", e),
            }
        }

        #[cfg(test)]
        mod test_has_data {
            use super::*;

            #[test]
            fn fail_on_no_dadta() {
                let mut state = RecordReaderState::new();
                state.record = Record::blank();
                match state.has_data() {
                    Err(ReadError::NoData) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn pass_on_data() {
                let mut state = RecordReaderState::new();
                state.record.data.push(DataArray::new("E", "RI"));
                let mut expected = Record::blank();
                expected.data.push(DataArray::new("E", "RI"));

                match state.has_data() {
                    Ok(s) => assert_eq!(s.record, expected),
                    e => panic!("{:?}", e),
                }
            }
        }

        #[cfg(test)]
        mod test_has_var {
            use super::*;

            #[test]
            fn fail_on_no_var() {
                let mut state = RecordReaderState::new();
                state.record = Record::blank();
                state.var_already_read = false;
                match state.has_var() {
                    Err(ReadError::NoIndependentVariable) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn pass_on_var() {
                let mut state = RecordReaderState::new();
                state.record = Record::blank();
                state.var_already_read = true;
                match state.has_var() {
                    Ok(s) => assert_eq!(s.record, Record::blank()),
                    e => panic!("{:?}", e),
                }
            }
        }

        #[cfg(test)]
        mod test_has_version {
            use super::*;

            #[test]
            fn fail_on_no_version() {
                let mut state = RecordReaderState::new();
                state.record = Record::blank();
                state.version_aready_read = false;
                match state.has_version() {
                    Err(ReadError::NoVersion) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn pass_on_name() {
                let mut state = RecordReaderState::new();
                state.record = Record::blank();
                state.version_aready_read = true;
                match state.has_version() {
                    Ok(s) => assert_eq!(s.record, Record::blank()),
                    e => panic!("{:?}", e),
                }
            }
        }

        #[cfg(test)]
        mod test_has_name {
            use super::*;

            #[test]
            fn fail_on_no_name() {
                let mut state = RecordReaderState::new();
                state.record = Record::blank();
                state.name_already_read = false;
                match state.has_name() {
                    Err(ReadError::NoName) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn pass_on_name() {
                let mut state = RecordReaderState::new();
                state.record = Record::blank();
                state.name_already_read = true;
                match state.has_name() {
                    Ok(s) => assert_eq!(s.record, Record::blank()),
                    e => panic!("{:?}", e),
                }
            }
        }

        #[cfg(test)]
        mod test_var_data_different_lengths {
            use super::*;

            #[test]
            fn pass_on_blank() {
                let mut state = RecordReaderState::new();
                state.record = Record::blank();
                match state.var_and_data_same_length() {
                    Ok(s) => assert_eq!(s.record, Record::blank()),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn pass_on_equal() {
                let mut state = RecordReaderState::new();
                state.record = Record::blank();
                state.record.data.push(DataArray {
                    name: String::new(),
                    format: String::new(),
                    samples: vec![Complex { re: 1., im: 1. }],
                });
                state.record.header.independent_variable.data = vec![1.];
                match state.clone().var_and_data_same_length() {
                    Ok(s) => assert_eq!(s.record, state.record),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn pass_on_var_zero_data_some() {
                let mut state = RecordReaderState::new();
                state.record = Record::blank();
                state.record.data.push(DataArray {
                    name: String::new(),
                    format: String::new(),
                    samples: vec![Complex { re: 1., im: 1. }],
                });
                state.record.header.independent_variable.data = vec![];
                match state.clone().var_and_data_same_length() {
                    Ok(s) => assert_eq!(s.record, state.record),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn fail_on_var_one_data_some() {
                let mut state = RecordReaderState::new();
                state.record = Record::blank();
                state.record.data.push(DataArray {
                    name: String::new(),
                    format: String::new(),
                    samples: vec![Complex { re: 1., im: 1. }, Complex { re: 1., im: 1. }],
                });
                state.record.header.independent_variable.data = vec![1.];
                match state.var_and_data_same_length() {
                    Err(ReadError::VarAndDataDifferentLengths(1, 2, 0)) => (),
                    e => panic!("{:?}", e),
                }
            }

            #[test]
            fn error_formatted_correctely() {
                let mut state = RecordReaderState::new();
                state.record = Record::blank();
                state.record.data.push(DataArray {
                    name: String::new(),
                    format: String::new(),
                    samples: vec![Complex { re: 1., im: 1. }],
                });
                state.record.data.push(DataArray {
                    name: String::new(),
                    format: String::new(),
                    samples: vec![Complex { re: 1., im: 1. }, Complex { re: 1., im: 1. }],
                });
                state.record.header.independent_variable.data = vec![1.];
                match state.var_and_data_same_length() {
                    Err(ReadError::VarAndDataDifferentLengths(1, 2, 1)) => (),
                    e => panic!("{:?}", e),
                }
            }
        }
    }
}
