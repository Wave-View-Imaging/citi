use citi::{
    assert_array_relative_eq, assert_complex_array_relative_eq, DataArray, Device, Record, Result,
    Var,
};
use num_complex::Complex;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

#[cfg(test)]
mod cti_read_regression_tests {
    use super::*;

    fn data_directory() -> PathBuf {
        let mut path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_buf.push("tests");
        path_buf.push("regression_files");
        path_buf
    }

    #[cfg(test)]
    mod test_read_display_memory_file {
        use super::*;

        fn filename() -> PathBuf {
            let mut path_buf = data_directory();
            path_buf.push("display_memory.cti");
            path_buf
        }

        fn setup() -> Result<Record> {
            let mut file = BufReader::new(File::open(filename()).unwrap());
            Record::read_from_source(&mut file)
        }

        #[test]
        fn name() {
            match setup() {
                Ok(file) => assert_eq!(file.header.name, "MEMORY"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn version() {
            match setup() {
                Ok(file) => assert_eq!(file.header.version, "A.01.00"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn comments() {
            match setup() {
                Ok(file) => assert_eq!(file.header.comments.len(), 0),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn constants() {
            match setup() {
                Ok(file) => assert_eq!(file.header.constants.len(), 0),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn devices() {
            match setup() {
                Ok(file) => {
                    assert_eq!(file.header.devices.len(), 1);
                    assert_eq!(file.header.devices[0].name, "NA");
                    assert_eq!(file.header.devices[0].entries.len(), 2);
                    assert_eq!(file.header.devices[0].entries[0], "VERSION HP8510B.05.00");
                    assert_eq!(file.header.devices[0].entries[1], "REGISTER 1");
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn independent_variable() {
            match setup() {
                Ok(file) => {
                    assert_eq!(file.header.independent_variable.name, "FREQ");
                    assert_eq!(file.header.independent_variable.format, "MAG");

                    let expected: Vec<f64> = vec![];
                    assert_array_relative_eq!(file.header.independent_variable.data, expected);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data() {
            match setup() {
                Ok(file) => {
                    let data = vec![
                        Complex {
                            re: -1.31189E-3,
                            im: -1.47980E-3,
                        },
                        Complex {
                            re: -3.67867E-3,
                            im: -0.67782E-3,
                        },
                        Complex {
                            re: -3.43990E-3,
                            im: 0.58746E-3,
                        },
                        Complex {
                            re: -2.70664E-4,
                            im: -9.76175E-4,
                        },
                        Complex {
                            re: 0.65892E-4,
                            im: -9.61571E-4,
                        },
                    ];

                    assert_eq!(file.data.len(), 1);
                    assert_eq!(file.data[0].name, "S");
                    assert_eq!(file.data[0].format, "RI");
                    assert_complex_array_relative_eq!(file.data[0].samples, data);
                }
                e => panic!("{:?}", e),
            }
        }
    }

    #[cfg(test)]
    mod test_read_data_file {
        use super::*;

        fn filename() -> PathBuf {
            let mut path_buf = data_directory();
            path_buf.push("data_file.cti");
            path_buf
        }

        fn setup() -> Result<Record> {
            let mut file = BufReader::new(File::open(filename()).unwrap());
            Record::read_from_source(&mut file)
        }

        #[test]
        fn name() {
            match setup() {
                Ok(file) => assert_eq!(file.header.name, "DATA"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn version() {
            match setup() {
                Ok(file) => assert_eq!(file.header.version, "A.01.00"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn comments() {
            match setup() {
                Ok(file) => assert_eq!(file.header.comments.len(), 0),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn constants() {
            match setup() {
                Ok(file) => assert_eq!(file.header.constants.len(), 0),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn devices() {
            match setup() {
                Ok(file) => {
                    assert_eq!(file.header.devices.len(), 1);
                    assert_eq!(file.header.devices[0].name, String::from("NA"));
                    assert_eq!(file.header.devices[0].entries.len(), 2);
                    assert_eq!(
                        file.header.devices[0].entries[0],
                        String::from("VERSION HP8510B.05.00")
                    );
                    assert_eq!(
                        file.header.devices[0].entries[1],
                        String::from("REGISTER 1")
                    );
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn independent_variable() {
            match setup() {
                Ok(file) => {
                    let expected: Vec<f64> = vec![
                        1000000000.,
                        1333333333.3333333,
                        1666666666.6666665,
                        2000000000.,
                        2333333333.3333333,
                        2666666666.6666665,
                        3000000000.,
                        3333333333.3333333,
                        3666666666.6666665,
                        4000000000.,
                    ];

                    assert_eq!(file.header.independent_variable.name, "FREQ");
                    assert_eq!(file.header.independent_variable.format, "MAG");

                    assert_array_relative_eq!(expected, file.header.independent_variable.data);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data() {
            match setup() {
                Ok(file) => {
                    let data = vec![
                        Complex {
                            re: 0.86303E-1,
                            im: -8.98651E-1,
                        },
                        Complex {
                            re: 8.97491E-1,
                            im: 3.06915E-1,
                        },
                        Complex {
                            re: -4.96887E-1,
                            im: 7.87323E-1,
                        },
                        Complex {
                            re: -5.65338E-1,
                            im: -7.05291E-1,
                        },
                        Complex {
                            re: 8.94287E-1,
                            im: -4.25537E-1,
                        },
                        Complex {
                            re: 1.77551E-1,
                            im: 8.96606E-1,
                        },
                        Complex {
                            re: -9.35028E-1,
                            im: -1.10504E-1,
                        },
                        Complex {
                            re: 3.69079E-1,
                            im: -9.13787E-1,
                        },
                        Complex {
                            re: 7.80120E-1,
                            im: 5.37841E-1,
                        },
                        Complex {
                            re: -7.78350E-1,
                            im: 5.72082E-1,
                        },
                    ];

                    assert_eq!(file.data.len(), 1);
                    assert_eq!(file.data[0].name, "S[1,1]");
                    assert_eq!(file.data[0].format, "RI");
                    assert_complex_array_relative_eq!(file.data[0].samples, data);
                }
                e => panic!("{:?}", e),
            }
        }
    }

    #[cfg(test)]
    mod test_wvi_file {
        use super::*;

        fn filename() -> PathBuf {
            let mut path_buf = data_directory();
            path_buf.push("wvi_file.cti");
            path_buf
        }

        fn setup() -> Result<Record> {
            let mut file = BufReader::new(File::open(filename()).unwrap());
            Record::read_from_source(&mut file)
        }

        #[test]
        fn name() {
            match setup() {
                Ok(file) => assert_eq!(file.header.name, "Antonly001"),
                Err(e) => panic!("File could not be read: {}", e),
            }
        }

        #[test]
        fn version() {
            match setup() {
                Ok(file) => assert_eq!(file.header.version, "A.01.01"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn comments() {
            let expected: Vec<String> = [
                "SOURCE: 10095059066467",
                "DATE: Fri, Jan 18, 2019, 14:14:44",
                "ANTPOS_TX: 28.4E-3 0E+0 -16E-3 90 270 0",
                "ANTPOS_RX: 28.4E-3 0E+0 -16E-3 90 270 0",
                "ANT_TX: NAH_003",
                "ANT_RX: NAH_003",
            ]
            .iter()
            .map(|&s| String::from(s))
            .collect();

            match setup() {
                Ok(file) => {
                    assert_eq!(file.header.comments, expected);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn constants() {
            match setup() {
                Ok(file) => assert_eq!(file.header.constants.len(), 0),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn devices() {
            match setup() {
                Ok(file) => {
                    assert_eq!(file.header.devices.len(), 0);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn independent_variable() {
            match setup() {
                Ok(file) => {
                    let expected: Vec<f64> = vec![100e6, 200e6];

                    assert_eq!(file.header.independent_variable.name, "Freq");
                    assert_eq!(file.header.independent_variable.format, "MAG");

                    assert_array_relative_eq!(expected, file.header.independent_variable.data);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data() {
            match setup() {
                Ok(file) => {
                    let data = vec![
                        Complex {
                            re: 8.609423041343689E-1,
                            im: 4.5087423920631409E-1,
                        },
                        Complex {
                            re: -6.1961996555328369E-1,
                            im: -7.2456854581832886E-1,
                        },
                    ];

                    assert_eq!(file.data.len(), 1);
                    assert_eq!(file.data[0].name, "S11");
                    assert_eq!(file.data[0].format, "RI");
                    assert_complex_array_relative_eq!(data, file.data[0].samples);
                }
                e => panic!("{:?}", e),
            }
        }
    }

    #[cfg(test)]
    mod test_read_list_cal_set {
        use super::*;

        fn filename() -> PathBuf {
            let mut path_buf = data_directory();
            path_buf.push("list_cal_set.cti");
            path_buf
        }

        fn setup() -> Result<Record> {
            let mut file = BufReader::new(File::open(filename()).unwrap());
            Record::read_from_source(&mut file)
        }

        #[test]
        fn name() {
            match setup() {
                Ok(file) => assert_eq!(file.header.name, "CAL_SET"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn version() {
            match setup() {
                Ok(file) => assert_eq!(file.header.version, "A.01.00"),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn comments() {
            match setup() {
                Ok(file) => assert_eq!(file.header.comments.len(), 0),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn constants() {
            match setup() {
                Ok(file) => assert_eq!(file.header.constants.len(), 0),
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn devices() {
            match setup() {
                Ok(file) => {
                    let device = Device {
                        name: String::from("NA"),
                        entries: vec![
                            "VERSION HP8510B.05.00",
                            "REGISTER 1",
                            "SWEEP_TIME 9.999987E-2",
                            "POWER1 1.0E1",
                            "POWER2 1.0E1",
                            "PARAMS 2",
                            "CAL_TYPE 3",
                            "POWER_SLOPE 0.0E0",
                            "SLOPE_MODE 0",
                            "TRIM_SWEEP 0",
                            "SWEEP_MODE 4",
                            "LOWPASS_FLAG -1",
                            "FREQ_INFO 1",
                            "SPAN 1000000000 3000000000 4",
                            "DUPLICATES 0",
                            "ARB_SEG 1000000000 1000000000 1",
                            "ARB_SEG 2000000000 3000000000 3",
                        ]
                        .iter()
                        .map(|&s| String::from(s))
                        .collect(),
                    };
                    let devices = vec![device];

                    assert_eq!(file.header.devices, devices);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn independent_variable() {
            match setup() {
                Ok(file) => {
                    let expected: Vec<f64> =
                        vec![1000000000., 2000000000., 2500000000., 3000000000.];

                    assert_eq!(file.header.independent_variable.name, "FREQ");
                    assert_eq!(file.header.independent_variable.format, "MAG");
                    assert_array_relative_eq!(expected, file.header.independent_variable.data);
                }
                e => panic!("{:?}", e),
            }
        }

        #[test]
        fn data() {
            match setup() {
                Ok(file) => {
                    let data0 = vec![
                        Complex {
                            re: 1.12134E-3,
                            im: 1.73103E-3,
                        },
                        Complex {
                            re: 4.23145E-3,
                            im: -5.36775E-3,
                        },
                        Complex {
                            re: -0.56815E-3,
                            im: 5.32650E-3,
                        },
                        Complex {
                            re: -1.85942E-3,
                            im: -4.07981E-3,
                        },
                    ];

                    let data1 = vec![
                        Complex {
                            re: 2.03895E-2,
                            im: -0.82674E-2,
                        },
                        Complex {
                            re: -4.21371E-2,
                            im: -0.24871E-2,
                        },
                        Complex {
                            re: 0.21038E-2,
                            im: -3.06778E-2,
                        },
                        Complex {
                            re: 1.20315E-2,
                            im: 5.99861E-2,
                        },
                    ];

                    let data2 = vec![
                        Complex {
                            re: 4.45404E-1,
                            im: 4.31518E-1,
                        },
                        Complex {
                            re: 8.34777E-1,
                            im: -1.33056E-1,
                        },
                        Complex {
                            re: -7.09137E-1,
                            im: 5.58410E-1,
                        },
                        Complex {
                            re: 4.84252E-1,
                            im: -8.07098E-1,
                        },
                    ];

                    assert_eq!(file.data.len(), 3);
                    assert_eq!(file.data[0].name, String::from("E[1]"));
                    assert_eq!(file.data[0].format, String::from("RI"));
                    assert_complex_array_relative_eq!(data0, file.data[0].samples);

                    assert_eq!(file.data[1].name, String::from("E[2]"));
                    assert_eq!(file.data[1].format, String::from("RI"));
                    assert_complex_array_relative_eq!(data1, file.data[1].samples);

                    assert_eq!(file.data[2].name, String::from("E[3]"));
                    assert_eq!(file.data[2].format, String::from("RI"));
                    assert_complex_array_relative_eq!(data2, file.data[2].samples);
                }
                e => panic!("{:?}", e),
            }
        }
    }
}

macro_rules! assert_files_equal {
    ($lhs:expr, $rhs:expr) => {
        let left = std::fs::read_to_string($lhs).expect("Unable to read file `$lhs`");
        let right = std::fs::read_to_string($rhs).expect("Unable to read file `$rhs`");

        assert_eq!(left, right);
    };
}

#[cfg(test)]
mod test_assert_files_equal {
    use super::*;

    fn data_directory() -> PathBuf {
        let mut path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_buf.push("tests");
        path_buf.push("regression_files");
        path_buf
    }

    fn filename1() -> PathBuf {
        let mut path_buf = data_directory();
        path_buf.push("display_memory.cti");
        path_buf
    }

    fn filename2() -> PathBuf {
        let mut path_buf = data_directory();
        path_buf.push("wvi_file.cti");
        path_buf
    }

    #[test]
    fn pass_on_same_file() {
        assert_files_equal!(filename1(), filename1());
    }

    #[test]
    #[should_panic]
    fn fail_on_different_files() {
        assert_files_equal!(filename1(), filename2());
    }

    #[test]
    #[should_panic]
    fn fail_on_bad_path() {
        assert_files_equal!(filename1(), "");
    }
}

#[cfg(test)]
mod cti_write_regression_tests {
    use super::*;
    use tempfile::tempdir;

    fn data_directory() -> PathBuf {
        let mut path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_buf.push("tests");
        path_buf.push("regression_files");
        path_buf
    }

    fn display_memory_filename() -> PathBuf {
        let mut path_buf = data_directory();
        path_buf.push("display_memory_output.cti");
        path_buf
    }

    fn display_memory_record() -> Record {
        let mut record = Record::new("A.01.00", "MEMORY");
        record.header.devices.push(Device {
            name: String::from("NA"),
            entries: vec!["VERSION HP8510B.05.00", "REGISTER 1"]
                .iter()
                .map(|&s| String::from(s))
                .collect(),
        });
        record.data.push(DataArray {
            name: String::from("S"),
            format: String::from("RI"),
            samples: vec![
                Complex::new(-1.31189E-3, -1.47980E-3),
                Complex::new(-3.67867E-3, -0.67782E-3),
                Complex::new(-3.43990E-3, 0.58746E-3),
                Complex::new(-2.70664E-4, -9.76175E-4),
                Complex::new(0.65892E-4, -9.61571E-4),
            ],
        });
        record.header.independent_variable = Var {
            name: String::from("FREQ"),
            format: String::from("MAG"),
            data: vec![0., 1., 2., 3., 4.],
        };

        record
    }

    #[test]
    fn display_memory() {
        let tmp = std::sync::Arc::new(tempdir().unwrap());
        let filename = tmp.path().join("temp-display-memory.cti");
        let mut file = File::create(filename.clone()).unwrap();
        let record = display_memory_record();
        record.write_to_sink(&mut file).unwrap();

        assert_files_equal!(display_memory_filename(), filename);
    }
}
