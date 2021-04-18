use lazy_static::lazy_static;
use regex::Regex;
use std::convert::TryFrom;
use std::str::FromStr;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum CTIParseError {
    #[error("Keyword `{0}` is not supported")]
    BadKeyword(String),
    #[error("Regex could not be parsed")]
    BadRegex,
    #[error("Cannot parse as number `{0}`")]
    NumberParseError(String),
}
// type CTIParseResult<T> = std::result::Result<T, CTIParseError>;

#[cfg(test)]
mod test_cti_parse_error {
    use super::*;

    mod test_display {
        use super::*;

        #[test]
        fn bad_keyword() {
            let error = CTIParseError::BadKeyword(String::from("asdf"));
            assert_eq!(format!("{}", error), "Keyword `asdf` is not supported");
        }

        #[test]
        fn bad_keyword_second() {
            let error = CTIParseError::BadKeyword(String::from("----"));
            assert_eq!(format!("{}", error), "Keyword `----` is not supported");
        }

        #[test]
        fn number_parse_error() {
            let error = CTIParseError::NumberParseError(String::from("asdf"));
            assert_eq!(format!("{}", error), "Cannot parse as number `asdf`");
        }

        #[test]
        fn number_parse_error_second() {
            let error = CTIParseError::NumberParseError(String::from("----"));
            assert_eq!(format!("{}", error), "Cannot parse as number `----`");
        }

        #[test]
        fn bad_regex() {
            let error = CTIParseError::BadRegex;
            assert_eq!(format!("{}", error), "Regex could not be parsed");
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CTIKeywords {
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

impl FromStr for CTIKeywords {
    type Err = CTIParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CTIKeywords::try_from(s)
    }
}

impl TryFrom<&str> for CTIKeywords {
    type Error = CTIParseError;

    fn try_from(line: &str) -> Result<Self, Self::Error> {
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
            "SEG_LIST_BEGIN" => Ok(CTIKeywords::SegListBegin),
            "SEG_LIST_END" => Ok(CTIKeywords::SegListEnd),
            "VAR_LIST_BEGIN" => Ok(CTIKeywords::VarListBegin),
            "VAR_LIST_END" => Ok(CTIKeywords::VarListEnd),
            "BEGIN" => Ok(CTIKeywords::Begin),
            "END" => Ok(CTIKeywords::End),
            _ if RE_DATA_PAIR.is_match(line) => {
                let cap = RE_DATA_PAIR.captures(line).ok_or(CTIParseError::BadRegex)?;
                Ok(CTIKeywords::DataPair{
                    real: cap.name("Real").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?.parse::<f64>().map_err(|_| CTIParseError::NumberParseError(String::from(line)))?,
                    imag: cap.name("Imag").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?.parse::<f64>().map_err(|_| CTIParseError::NumberParseError(String::from(line)))?,
                })
            },
            _ if RE_DEVICE.is_match(line) => {
                let cap = RE_DEVICE.captures(line).ok_or(CTIParseError::BadRegex)?;
                Ok(CTIKeywords::Device{
                    name: String::from(cap.name("Name").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?),
                    value: String::from(cap.name("Value").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?),
                })
            },
            _ if RE_SEG_ITEM.is_match(line) => {
                let cap = RE_SEG_ITEM.captures(line).ok_or(CTIParseError::BadRegex)?;
                Ok(CTIKeywords::SegItem{
                    first: cap.name("First").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?.parse::<f64>().map_err(|_| CTIParseError::NumberParseError(String::from(line)))?,
                    last: cap.name("Last").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?.parse::<f64>().map_err(|_| CTIParseError::NumberParseError(String::from(line)))?,
                    number: cap.name("Number").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?.parse::<usize>().map_err(|_| CTIParseError::NumberParseError(String::from(line)))?,
                })
            },
            _ if RE_VAR_ITEM.is_match(line) => {
                let cap = RE_VAR_ITEM.captures(line).ok_or(CTIParseError::BadRegex)?;
                Ok(CTIKeywords::VarListItem(
                    cap.name("Value").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?.parse::<f64>().map_err(|_| CTIParseError::NumberParseError(String::from(line)))?
                ))
            },
            _ if RE_DATA.is_match(line) => {
                let cap = RE_DATA.captures(line).ok_or(CTIParseError::BadRegex)?;
                Ok(CTIKeywords::Data{
                    name: String::from(cap.name("Name").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?),
                    format: String::from(cap.name("Format").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?),
                })
            },
            _ if RE_VAR.is_match(line) => {
                let cap = RE_VAR.captures(line).ok_or(CTIParseError::BadRegex)?;
                let closure = |m: String| {if m.is_empty() {None} else {Some(m)}};
                Ok(CTIKeywords::Var{
                    name: String::from(cap.name("Name").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?),
                    format: closure(cap.name("Format").map(|m| String::from(m.as_str())).ok_or(CTIParseError::BadRegex)?),
                    length: cap.name("Length").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?.parse::<usize>().map_err(|_| CTIParseError::NumberParseError(String::from(line)))?,
                })
            },
            _ if RE_COMMENT.is_match(line) => {
                let cap = RE_COMMENT.captures(line).ok_or(CTIParseError::BadRegex)?;
                Ok(CTIKeywords::Comment(String::from(cap.name("Comment").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?)))
            },
            _ if RE_CITIFILE.is_match(line) => {
                let cap = RE_CITIFILE.captures(line).ok_or(CTIParseError::BadRegex)?;
                Ok(CTIKeywords::CITIFile{
                    version: String::from(cap.name("Version").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?)
                })
            },
            _ if RE_NAME.is_match(line) => {
                let cap = RE_NAME.captures(line).ok_or(CTIParseError::BadRegex)?;
                Ok(CTIKeywords::Name(String::from(cap.name("Name").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?)))
            },
            _ if RE_CONSTANT.is_match(line) => {
                let cap = RE_CONSTANT.captures(line).ok_or(CTIParseError::BadRegex)?;
                Ok(CTIKeywords::Constant{
                    name: String::from(cap.name("Name").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?),
                    value: String::from(cap.name("Value").map(|m| m.as_str()).ok_or(CTIParseError::BadRegex)?)
                })
            },
            _ => Err(CTIParseError::BadKeyword(String::from(line))),
        }
    }
}

impl fmt::Display for CTIKeywords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CTIKeywords::CITIFile{version} => write!(f, "CITIFILE {}", version),
            CTIKeywords::Name(name) => write!(f, "NAME {}", name),
            CTIKeywords::Var{name, format, length} => match format {
                Some(form) => write!(f, "VAR {} {} {}", name, form, length),
                None => write!(f, "VAR {} {}", name, length),
            },
            CTIKeywords::Constant{name, value} => write!(f, "CONSTANT {} {}", name, value),
            CTIKeywords::Device{name, value} => write!(f, "#{} {}", name, value),
            CTIKeywords::SegListBegin => write!(f, "SEG_LIST_BEGIN"),
            CTIKeywords::SegItem{first, last, number} => write!(f, "SEG {} {} {}", first, last, number),
            CTIKeywords::SegListEnd => write!(f, "SEG_LIST_END"),
            CTIKeywords::VarListBegin => write!(f, "VAR_LIST_BEGIN"),
            CTIKeywords::VarListItem(n) => write!(f, "{}", n),
            CTIKeywords::VarListEnd => write!(f, "VAR_LIST_END"),
            CTIKeywords::Data{name, format} => write!(f, "DATA {} {}", name, format),
            CTIKeywords::DataPair{real, imag} => write!(f, "{:E},{:E}", real, imag),
            CTIKeywords::Begin => write!(f, "BEGIN"),
            CTIKeywords::End => write!(f, "END"),
            CTIKeywords::Comment(comment) => write!(f, "!{}", comment),
        }
    }
}

#[cfg(test)]
mod test_cti_keywords {
    use super::*;

    #[cfg(test)]
    mod test_fmt_display {
        use super::*;

        #[test]
        fn citifile_a_01_00() {
            let keyword = CTIKeywords::CITIFile{version: String::from("A.01.00")};
            assert_eq!("CITIFILE A.01.00", format!("{}", keyword));
        }

        #[test]
        fn citifile_a_01_01() {
            let keyword = CTIKeywords::CITIFile{version: String::from("A.01.01")};
            assert_eq!("CITIFILE A.01.01", format!("{}", keyword));
        }

        #[test]
        fn name() {
            let keyword = CTIKeywords::Name(String::from("CAL_SET"));
            assert_eq!("NAME CAL_SET", format!("{}", keyword));
        }

        #[test]
        fn var_standard() {
            let keyword = CTIKeywords::Var{name: String::from("FREQ"), format: Some(String::from("MAG")), length: 201};
            assert_eq!("VAR FREQ MAG 201", format!("{}", keyword));
        }

        #[test]
        fn var_no_format() {
            let keyword = CTIKeywords::Var{name: String::from("FREQ"), format: None, length: 202};
            assert_eq!("VAR FREQ 202", format!("{}", keyword));
        }

        #[test]
        fn constant() {
            let keyword = CTIKeywords::Constant{name: String::from("A_CONSTANT"), value: String::from("1.2345")};
            assert_eq!("CONSTANT A_CONSTANT 1.2345", format!("{}", keyword));
        }

        #[test]
        fn device() {
            let keyword = CTIKeywords::Device{name: String::from("NA"), value: String::from("REGISTER 1")};
            assert_eq!("#NA REGISTER 1", format!("{}", keyword));
        }

        #[test]
        fn device_number() {
            let keyword = CTIKeywords::Device{name: String::from("NA"), value: String::from("POWER2 1.0E1")};
            assert_eq!("#NA POWER2 1.0E1", format!("{}", keyword));
        }

        #[test]
        fn device_name() {
            let keyword = CTIKeywords::Device{name: String::from("WVI"), value: String::from("A B")};
            assert_eq!("#WVI A B", format!("{}", keyword));
        }

        #[test]
        fn seg_list_begin() {
            let keyword = CTIKeywords::SegListBegin;
            assert_eq!("SEG_LIST_BEGIN", format!("{}", keyword));
        }

        #[test]
        fn seg_item() {
            let keyword = CTIKeywords::SegItem{first: 1000000000., last: 4000000000., number: 10};
            assert_eq!("SEG 1000000000 4000000000 10", format!("{}", keyword));
        }

        #[test]
        fn seg_list_end() {
            let keyword = CTIKeywords::SegListEnd;
            assert_eq!("SEG_LIST_END", format!("{}", keyword));
        }

        #[test]
        fn var_list_begin() {
            let keyword = CTIKeywords::VarListBegin;
            assert_eq!("VAR_LIST_BEGIN", format!("{}", keyword));
        }

        #[test]
        fn var_item() {
            let keyword = CTIKeywords::VarListItem(100000.);
            assert_eq!("100000", format!("{}", keyword));
        }

        #[test]
        fn var_list_end() {
            let keyword = CTIKeywords::VarListEnd;
            assert_eq!("VAR_LIST_END", format!("{}", keyword));
        }

        #[test]
        fn data_s11() {
            let keyword = CTIKeywords::Data{name: String::from("S[1,1]"), format: String::from("RI")};
            assert_eq!("DATA S[1,1] RI", format!("{}", keyword));
        }

        #[test]
        fn data_e() {
            let keyword = CTIKeywords::Data{name: String::from("E"), format: String::from("RI")};
            assert_eq!("DATA E RI", format!("{}", keyword));
        }

        #[test]
        fn data_pair_simple() {
            let keyword = CTIKeywords::DataPair{real: 1e9, imag: -1e9};
            assert_eq!("1E9,-1E9", format!("{}", keyword));
        }
        
        #[test]
        fn data_pair() {
            let keyword = CTIKeywords::DataPair{real: 0.86303e-1, imag: -8.98651e-1};
            assert_eq!("8.6303E-2,-8.98651E-1", format!("{}", keyword));
        }

        #[test]
        fn begin() {
            let keyword = CTIKeywords::Begin;
            assert_eq!("BEGIN", format!("{}", keyword));
        }

        #[test]
        fn end() {
            let keyword = CTIKeywords::End;
            assert_eq!("END", format!("{}", keyword));
        }

        #[test]
        fn comment() {
            let keyword = CTIKeywords::Comment(String::from("DATE: 2019.11.01"));
            assert_eq!("!DATE: 2019.11.01", format!("{}", keyword));
        }
    }

    #[cfg(test)]
    mod test_from_str_slice {
        use super::*;
        use approx::*;

        #[test]
        fn fails_on_bad_string() {
            let keyword = CTIKeywords::from_str("this is a bad string");
            assert_eq!(keyword, Err(CTIParseError::BadKeyword(String::from("this is a bad string"))));
        }

        #[test]
        fn citifile_a_01_00() {
            let keyword = CTIKeywords::from_str("CITIFILE A.01.00");
            assert_eq!(keyword, Ok(CTIKeywords::CITIFile{version: String::from("A.01.00")}));
        }

        #[test]
        fn citifile_a_01_01() {
            let keyword = CTIKeywords::from_str("CITIFILE A.01.01");
            assert_eq!(keyword, Ok(CTIKeywords::CITIFile{version: String::from("A.01.01")}));
        }

        #[test]
        fn name_cal_set() {
            let keyword = CTIKeywords::from_str("NAME CAL_SET");
            assert_eq!(keyword, Ok(CTIKeywords::Name(String::from("CAL_SET"))));
        }

        #[test]
        fn name_raw_data() {
            let keyword = CTIKeywords::from_str("NAME RAW_DATA");
            assert_eq!(keyword, Ok(CTIKeywords::Name(String::from("RAW_DATA"))));
        }

        #[test]
        fn constant() {
            let keyword = CTIKeywords::from_str("CONSTANT A_CONSTANT 1.2345");
            assert_eq!(keyword, Ok(CTIKeywords::Constant{name: String::from("A_CONSTANT"), value: String::from("1.2345")}));
        }

        #[test]
        fn device() {
            let keyword = CTIKeywords::from_str("#NA REGISTER 1");
            assert_eq!(keyword, Ok(CTIKeywords::Device{name: String::from("NA"), value: String::from("REGISTER 1")}));
        }

        #[test]
        fn device_number() {
            let keyword = CTIKeywords::from_str("#NA POWER2 1.0E1");
            assert_eq!(keyword, Ok(CTIKeywords::Device{name: String::from("NA"), value: String::from("POWER2 1.0E1")}));
        }

        #[test]
        fn device_name() {
            let keyword = CTIKeywords::from_str("#WVI A B");
            assert_eq!(keyword, Ok(CTIKeywords::Device{name: String::from("WVI"), value: String::from("A B")}));
        }

        #[test]
        fn var_standard() {
            let keyword = CTIKeywords::from_str("VAR FREQ MAG 201");
            assert_eq!(keyword, Ok(CTIKeywords::Var{name: String::from("FREQ"), format: Some(String::from("MAG")), length: 201}));
        }

        #[test]
        fn var_no_format() {
            let keyword = CTIKeywords::from_str("VAR FREQ 202");
            assert_eq!(keyword, Ok(CTIKeywords::Var{name: String::from("FREQ"), format: None, length: 202}));
        }

        #[test]
        fn seg_list_begin() {
            let keyword = CTIKeywords::from_str("SEG_LIST_BEGIN");
            assert_eq!(keyword, Ok(CTIKeywords::SegListBegin));
        }

        #[test]
        fn seg_item() {
            let keyword = CTIKeywords::from_str("SEG 1000000000 4000000000 10");
            match keyword {
                Ok(CTIKeywords::SegItem{first, last, number}) => {
                    assert_relative_eq!(first, 1000000000.);
                    assert_relative_eq!(last, 4000000000.);
                    assert_eq!(number, 10);
                },
                _ => panic!()
            }
        }

        #[test]
        fn seg_item_exponential() {
            let keyword = CTIKeywords::from_str("SEG 1e9 1E4 100");
            match keyword {
                Ok(CTIKeywords::SegItem{first, last, number}) => {
                    assert_relative_eq!(first, 1e9);
                    assert_relative_eq!(last, 1e4);
                    assert_eq!(number, 100);
                },
                _ => panic!()
            }
        }

        #[test]
        fn seg_item_negative() {
            let keyword = CTIKeywords::from_str("SEG -1e9 1E-4 1");
            match keyword {
                Ok(CTIKeywords::SegItem{first, last, number}) => {
                    assert_relative_eq!(first, -1e9);
                    assert_relative_eq!(last, 1e-4);
                    assert_eq!(number, 1);
                },
                _ => panic!()
            }
        }

        #[test]
        fn seg_list_end() {
            let keyword = CTIKeywords::from_str("SEG_LIST_END");
            assert_eq!(keyword, Ok(CTIKeywords::SegListEnd));
        }

        #[test]
        fn var_list_begin() {
            let keyword = CTIKeywords::from_str("VAR_LIST_BEGIN");
            assert_eq!(keyword, Ok(CTIKeywords::VarListBegin));
        }

        #[test]
        fn var_item() {
            let keyword = CTIKeywords::from_str("100000");
            match keyword {
                Ok(CTIKeywords::VarListItem(value)) => {
                    assert_relative_eq!(value, 100000.);
                },
                _ => panic!()
            }
        }

        #[test]
        fn var_item_exponential() {
            let keyword = CTIKeywords::from_str("100E+6");
            match keyword {
                Ok(CTIKeywords::VarListItem(value)) => {
                    assert_relative_eq!(value, 100E+6);
                },
                _ => panic!()
            }
        }

        #[test]
        fn var_item_negative_exponential() {
            let keyword = CTIKeywords::from_str("-1e-2");
            match keyword {
                Ok(CTIKeywords::VarListItem(value)) => {
                    assert_relative_eq!(value, -1e-2);
                },
                _ => panic!()
            }
        }

        #[test]
        fn var_item_negative() {
            let keyword = CTIKeywords::from_str("-100000");
            match keyword {
                Ok(CTIKeywords::VarListItem(value)) => {
                    assert_relative_eq!(value, -100000.);
                },
                _ => panic!()
            }
        }

        #[test]
        fn var_list_end() {
            let keyword = CTIKeywords::from_str("VAR_LIST_END");
            assert_eq!(keyword, Ok(CTIKeywords::VarListEnd));
        }

        #[test]
        fn data_s11() {
            let keyword = CTIKeywords::from_str("DATA S[1,1] RI");
            assert_eq!(keyword, Ok(CTIKeywords::Data{name: String::from("S[1,1]"), format: String::from("RI")}));
        }

        #[test]
        fn data_e() {
            let keyword = CTIKeywords::from_str("DATA E RI");
            assert_eq!(keyword, Ok(CTIKeywords::Data{name: String::from("E"), format: String::from("RI")}));
        }

        #[test]
        fn data_pair_simple() {
            let keyword = CTIKeywords::from_str("1E9,-1E9");
            match keyword {
                Ok(CTIKeywords::DataPair{real, imag}) => {
                    assert_relative_eq!(real, 1e9);
                    assert_relative_eq!(imag, -1e9);
                },
                _ => panic!()
            }
        }

        #[test]
        fn data_pair() {
            let keyword = CTIKeywords::from_str("8.6303E-2,-8.98651E-1");
            match keyword {
                Ok(CTIKeywords::DataPair{real, imag}) => {
                    assert_relative_eq!(real, 0.86303e-1);
                    assert_relative_eq!(imag, -8.98651e-1);
                },
                _ => panic!()
            }
        }

        #[test]
        fn data_pair_spaced() {
            let keyword = CTIKeywords::from_str("8.6303E-2, -8.98651E-1");
            match keyword {
                Ok(CTIKeywords::DataPair{real, imag}) => {
                    assert_relative_eq!(real, 0.86303e-1);
                    assert_relative_eq!(imag, -8.98651e-1);
                },
                _ => panic!()
            }
        }

        #[test]
        fn begin() {
            let keyword = CTIKeywords::from_str("BEGIN");
            assert_eq!(keyword, Ok(CTIKeywords::Begin));
        }

        #[test]
        fn end() {
            let keyword = CTIKeywords::from_str("END");
            assert_eq!(keyword, Ok(CTIKeywords::End));
        }

        #[test]
        fn comment() {
            let keyword = CTIKeywords::from_str("!DATE: 2019.11.01");
            assert_eq!(keyword, Ok(CTIKeywords::Comment(String::from("DATE: 2019.11.01"))));
        }
    }

    #[cfg(test)]
    mod test_try_from_string_slice {
        use super::*;
        use approx::*;

        #[test]
        fn fails_on_bad_string() {
            let keyword = CTIKeywords::try_from("this is a bad string");
            assert_eq!(keyword, Err(CTIParseError::BadKeyword(String::from("this is a bad string"))));
        }

        #[test]
        fn citifile_a_01_00() {
            let keyword = CTIKeywords::try_from("CITIFILE A.01.00");
            assert_eq!(keyword, Ok(CTIKeywords::CITIFile{version: String::from("A.01.00")}));
        }

        #[test]
        fn citifile_a_01_01() {
            let keyword = CTIKeywords::try_from("CITIFILE A.01.01");
            assert_eq!(keyword, Ok(CTIKeywords::CITIFile{version: String::from("A.01.01")}));
        }

        #[test]
        fn name_cal_set() {
            let keyword = CTIKeywords::try_from("NAME CAL_SET");
            assert_eq!(keyword, Ok(CTIKeywords::Name(String::from("CAL_SET"))));
        }

        #[test]
        fn name_raw_data() {
            let keyword = CTIKeywords::try_from("NAME RAW_DATA");
            assert_eq!(keyword, Ok(CTIKeywords::Name(String::from("RAW_DATA"))));
        }

        #[test]
        fn constant() {
            let keyword = CTIKeywords::try_from("CONSTANT A_CONSTANT 1.2345");
            assert_eq!(keyword, Ok(CTIKeywords::Constant{name: String::from("A_CONSTANT"), value: String::from("1.2345")}));
        }

        #[test]
        fn device() {
            let keyword = CTIKeywords::try_from("#NA REGISTER 1");
            assert_eq!(keyword, Ok(CTIKeywords::Device{name: String::from("NA"), value: String::from("REGISTER 1")}));
        }

        #[test]
        fn device_number() {
            let keyword = CTIKeywords::try_from("#NA POWER2 1.0E1");
            assert_eq!(keyword, Ok(CTIKeywords::Device{name: String::from("NA"), value: String::from("POWER2 1.0E1")}));
        }

        #[test]
        fn device_name() {
            let keyword = CTIKeywords::try_from("#WVI A B");
            assert_eq!(keyword, Ok(CTIKeywords::Device{name: String::from("WVI"), value: String::from("A B")}));
        }

        #[test]
        fn var_standard() {
            let keyword = CTIKeywords::try_from("VAR FREQ MAG 201");
            assert_eq!(keyword, Ok(CTIKeywords::Var{name: String::from("FREQ"), format: Some(String::from("MAG")), length: 201}));
        }

        #[test]
        fn var_no_format() {
            let keyword = CTIKeywords::try_from("VAR FREQ 202");
            assert_eq!(keyword, Ok(CTIKeywords::Var{name: String::from("FREQ"), format: None, length: 202}));
        }

        #[test]
        fn seg_list_begin() {
            let keyword = CTIKeywords::try_from("SEG_LIST_BEGIN");
            assert_eq!(keyword, Ok(CTIKeywords::SegListBegin));
        }

        #[test]
        fn seg_item() {
            let keyword = CTIKeywords::try_from("SEG 1000000000 4000000000 10");
            match keyword {
                Ok(CTIKeywords::SegItem{first, last, number}) => {
                    assert_relative_eq!(first, 1000000000.);
                    assert_relative_eq!(last, 4000000000.);
                    assert_eq!(number, 10);
                },
                _ => panic!()
            }
        }

        #[test]
        fn seg_item_exponential() {
            let keyword = CTIKeywords::try_from("SEG 1e9 1E4 100");
            match keyword {
                Ok(CTIKeywords::SegItem{first, last, number}) => {
                    assert_relative_eq!(first, 1e9);
                    assert_relative_eq!(last, 1e4);
                    assert_eq!(number, 100);
                },
                _ => panic!()
            }
        }

        #[test]
        fn seg_item_negative() {
            let keyword = CTIKeywords::try_from("SEG -1e9 1E-4 1");
            match keyword {
                Ok(CTIKeywords::SegItem{first, last, number}) => {
                    assert_relative_eq!(first, -1e9);
                    assert_relative_eq!(last, 1e-4);
                    assert_eq!(number, 1);
                },
                _ => panic!()
            }
        }

        #[test]
        fn seg_list_end() {
            let keyword = CTIKeywords::try_from("SEG_LIST_END");
            assert_eq!(keyword, Ok(CTIKeywords::SegListEnd));
        }

        #[test]
        fn var_list_begin() {
            let keyword = CTIKeywords::try_from("VAR_LIST_BEGIN");
            assert_eq!(keyword, Ok(CTIKeywords::VarListBegin));
        }

        #[test]
        fn var_item() {
            let keyword = CTIKeywords::try_from("100000");
            match keyword {
                Ok(CTIKeywords::VarListItem(value)) => {
                    assert_relative_eq!(value, 100000.);
                },
                _ => panic!()
            }
        }

        #[test]
        fn var_item_exponential() {
            let keyword = CTIKeywords::try_from("100E+6");
            match keyword {
                Ok(CTIKeywords::VarListItem(value)) => {
                    assert_relative_eq!(value, 100E+6);
                },
                _ => panic!()
            }
        }

        #[test]
        fn var_item_negative_exponential() {
            let keyword = CTIKeywords::try_from("-1e-2");
            match keyword {
                Ok(CTIKeywords::VarListItem(value)) => {
                    assert_relative_eq!(value, -1e-2);
                },
                _ => panic!()
            }
        }

        #[test]
        fn var_item_negative() {
            let keyword = CTIKeywords::try_from("-100000");
            match keyword {
                Ok(CTIKeywords::VarListItem(value)) => {
                    assert_relative_eq!(value, -100000.);
                },
                _ => panic!()
            }
        }

        #[test]
        fn var_list_end() {
            let keyword = CTIKeywords::try_from("VAR_LIST_END");
            assert_eq!(keyword, Ok(CTIKeywords::VarListEnd));
        }

        #[test]
        fn data_s11() {
            let keyword = CTIKeywords::try_from("DATA S[1,1] RI");
            assert_eq!(keyword, Ok(CTIKeywords::Data{name: String::from("S[1,1]"), format: String::from("RI")}));
        }

        #[test]
        fn data_e() {
            let keyword = CTIKeywords::try_from("DATA E RI");
            assert_eq!(keyword, Ok(CTIKeywords::Data{name: String::from("E"), format: String::from("RI")}));
        }

        #[test]
        fn data_pair_simple() {
            let keyword = CTIKeywords::try_from("1E9,-1E9");
            match keyword {
                Ok(CTIKeywords::DataPair{real, imag}) => {
                    assert_relative_eq!(real, 1e9);
                    assert_relative_eq!(imag, -1e9);
                },
                _ => panic!()
            }
        }

        #[test]
        fn data_pair() {
            let keyword = CTIKeywords::try_from("8.6303E-2,-8.98651E-1");
            match keyword {
                Ok(CTIKeywords::DataPair{real, imag}) => {
                    assert_relative_eq!(real, 0.86303e-1);
                    assert_relative_eq!(imag, -8.98651e-1);
                },
                _ => panic!()
            }
        }

        #[test]
        fn data_pair_spaced() {
            let keyword = CTIKeywords::try_from("8.6303E-2, -8.98651E-1");
            match keyword {
                Ok(CTIKeywords::DataPair{real, imag}) => {
                    assert_relative_eq!(real, 0.86303e-1);
                    assert_relative_eq!(imag, -8.98651e-1);
                },
                _ => panic!()
            }
        }

        #[test]
        fn begin() {
            let keyword = CTIKeywords::try_from("BEGIN");
            assert_eq!(keyword, Ok(CTIKeywords::Begin));
        }

        #[test]
        fn end() {
            let keyword = CTIKeywords::try_from("END");
            assert_eq!(keyword, Ok(CTIKeywords::End));
        }

        #[test]
        fn comment() {
            let keyword = CTIKeywords::try_from("!DATE: 2019.11.01");
            assert_eq!(keyword, Ok(CTIKeywords::Comment(String::from("DATE: 2019.11.01"))));
        }
    }
}

