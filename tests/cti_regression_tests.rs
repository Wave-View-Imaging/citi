use reco::io::cti::{CTIResult, CTIFile, CTIDevice, CTIDevices};
use std::path::PathBuf;

macro_rules! assert_array_relative_eq {
    ($lhs:expr, $rhs:expr) => {
        // Size
        assert_eq!($lhs.len(), $rhs.len());

        // Values
        let it = $lhs.iter().zip($rhs.iter());
        for (l, r) in it {
            approx::assert_relative_eq!(l, r);
        }
    };
}

#[cfg(test)]
mod test_assert_array_relative_eq_macro {
    #[test]
    fn pass_on_empty() {
        let expected: Vec<f64> = vec![];
        let result: Vec<f64> = vec![];
        assert_array_relative_eq!(expected, result);
    }

    #[test]
    fn pass_on_same() {
        let expected: Vec<f64> = vec![1., 2., 3.];
        let result: Vec<f64> = vec![1., 2., 3.];
        assert_array_relative_eq!(expected, result);
    }

    #[test]
    #[should_panic]
    fn fail_on_different() {
        let expected: Vec<f64> = vec![1., 2., 3.];
        let result: Vec<f64> = vec![1., 2., 0.];
        assert_array_relative_eq!(expected, result);
    }

    #[test]
    #[should_panic]
    fn fail_on_different_size() {
        let expected: Vec<f64> = vec![1., 2., 3.];
        let result: Vec<f64> = vec![1., 2.];
        assert_array_relative_eq!(expected, result);
    }
}

#[cfg(test)]
mod cti_regression_tests {
    use super::*;

    fn data_directory() -> PathBuf {
        let mut path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path_buf.push("tests");
        path_buf.push("cti_regression_files");
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

        fn setup() -> CTIResult<CTIFile> {
            CTIFile::read(&filename())
        }

        #[test]
        fn name() {
            match setup() {
                Ok(file) => assert_eq!(file.header.name, Some(String::from("MEMORY"))),
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn version() {
            match setup() {
                Ok(file) => assert_eq!(file.header.version, Some(String::from("A.01.00"))),
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn comments() {
            match setup() {
                Ok(file) => assert_eq!(file.header.comments.len(), 0),
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn constants() {
            match setup() {
                Ok(file) => assert_eq!(file.header.constants.len(), 0),
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn devices() {
            match setup() {
                Ok(file) => {
                    assert_eq!(file.header.devices.devices.len(), 1);
                    assert_eq!(file.header.devices.devices[0].name, String::from("NA"));
                    assert_eq!(file.header.devices.devices[0].entries.len(), 2);
                    assert_eq!(file.header.devices.devices[0].entries[0], String::from("VERSION HP8510B.05.00"));
                    assert_eq!(file.header.devices.devices[0].entries[1], String::from("REGISTER 1"));
                },
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn independent_variable() {
            match setup() {
                Ok(file) => {
                    assert_eq!(file.header.independent_variable.name, Some(String::from("FREQ")));
                    assert_eq!(file.header.independent_variable.format, Some(String::from("MAG")));

                    let expected: Vec<f64> = vec![];
                    assert_array_relative_eq!(file.header.independent_variable.data, expected);
                },
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn data() {
            match setup() {
                Ok(file) => {
                    let real: Vec<f64> = vec![-1.31189E-3, -3.67867E-3, -3.43990E-3, -2.70664E-4, 0.65892E-4];
                    let imag: Vec<f64> = vec![-1.47980E-3, -0.67782E-3, 0.58746E-3, -9.76175E-4, -9.61571E-4];

                    assert_eq!(file.data.len(), 1);
                    assert_eq!(file.data[0].name, Some(String::from("S")));
                    assert_eq!(file.data[0].format, Some(String::from("RI")));

                    assert_array_relative_eq!(real, file.data[0].real);
                    assert_array_relative_eq!(imag, file.data[0].imag);
                },
                Err(_) => panic!("File could not be read"),
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

        fn setup() -> CTIResult<CTIFile> {
            CTIFile::read(&filename())
        }

        #[test]
        fn name() {
            match setup() {
                Ok(file) => assert_eq!(file.header.name, Some(String::from("DATA"))),
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn version() {
            match setup() {
                Ok(file) => assert_eq!(file.header.version, Some(String::from("A.01.00"))),
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn comments() {
            match setup() {
                Ok(file) => assert_eq!(file.header.comments.len(), 0),
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn constants() {
            match setup() {
                Ok(file) => assert_eq!(file.header.constants.len(), 0),
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn devices() {
            match setup() {
                Ok(file) => {
                    assert_eq!(file.header.devices.devices.len(), 1);
                    assert_eq!(file.header.devices.devices[0].name, String::from("NA"));
                    assert_eq!(file.header.devices.devices[0].entries.len(), 2);
                    assert_eq!(file.header.devices.devices[0].entries[0], String::from("VERSION HP8510B.05.00"));
                    assert_eq!(file.header.devices.devices[0].entries[1], String::from("REGISTER 1"));
                },
                Err(_) => panic!("File could not be read"),
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

                    assert_eq!(file.header.independent_variable.name, Some(String::from("FREQ")));
                    assert_eq!(file.header.independent_variable.format, Some(String::from("MAG")));
                
                    assert_array_relative_eq!(expected, file.header.independent_variable.data);
                },
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn data() {
            match setup() {
                Ok(file) => {
                    let real: Vec<f64> = vec![
                        0.86303E-1, 8.97491E-1, -4.96887E-1, -5.65338E-1, 8.94287E-1,
                        1.77551E-1, -9.35028E-1, 3.69079E-1, 7.80120E-1, -7.78350E-1
                    ];
                    let imag: Vec<f64> = vec![
                        -8.98651E-1, 3.06915E-1, 7.87323E-1, -7.05291E-1, -4.25537E-1,
                        8.96606E-1, -1.10504E-1, -9.13787E-1, 5.37841E-1, 5.72082E-1
                    ];

                    assert_eq!(file.data.len(), 1);
                    assert_eq!(file.data[0].name, Some(String::from("S[1,1]")));
                    assert_eq!(file.data[0].format, Some(String::from("RI")));

                    assert_array_relative_eq!(real, file.data[0].real);
                    assert_array_relative_eq!(imag, file.data[0].imag);
                },
                Err(_) => panic!("File could not be read"),
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

        fn setup() -> CTIResult<CTIFile> {
            CTIFile::read(&filename())
        }

        #[test]
        fn name() {
            match setup() {
                Ok(file) => assert_eq!(file.header.name, Some(String::from("Antonly001"))),
                Err(e) => panic!("File could not be read: {}", e),
            }
        }

        #[test]
        fn version() {
            match setup() {
                Ok(file) => assert_eq!(file.header.version, Some(String::from("A.01.01"))),
                Err(_) => panic!("File could not be read"),
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
            ].iter().map(|&s| String::from(s)).collect(); 

            match setup() {
                Ok(file) => {
                    assert_eq!(file.header.comments, expected);
                },
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn constants() {
            match setup() {
                Ok(file) => assert_eq!(file.header.constants.len(), 0),
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn devices() {
            match setup() {
                Ok(file) => {
                    assert_eq!(file.header.devices.devices, vec![]);
                },
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn independent_variable() {
            match setup() {
                Ok(file) => {
                    let expected: Vec<f64> = vec![
                        100e6,
                        200e6,
                    ];

                    assert_eq!(file.header.independent_variable.name, Some(String::from("Freq")));
                    assert_eq!(file.header.independent_variable.format, Some(String::from("MAG")));
                
                    assert_array_relative_eq!(expected, file.header.independent_variable.data);
                },
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn data() {
            match setup() {
                Ok(file) => {
                    let real: Vec<f64> = vec![8.609423041343689E-1, -6.1961996555328369E-1];
                    let imag: Vec<f64> = vec![4.5087423920631409E-1, -7.2456854581832886E-1];

                    assert_eq!(file.data.len(), 1);
                    assert_eq!(file.data[0].name, Some(String::from("S11")));
                    assert_eq!(file.data[0].format, Some(String::from("RI")));
                    assert_array_relative_eq!(real, file.data[0].real);
                    assert_array_relative_eq!(imag, file.data[0].imag);
                },
                Err(_) => panic!("File could not be read"),
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

        fn setup() -> CTIResult<CTIFile> {
            CTIFile::read(&filename())
        }

        #[test]
        fn name() {
            match setup() {
                Ok(file) => assert_eq!(file.header.name, Some(String::from("CAL_SET"))),
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn version() {
            match setup() {
                Ok(file) => assert_eq!(file.header.version, Some(String::from("A.01.00"))),
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn comments() {
            match setup() {
                Ok(file) => assert_eq!(file.header.comments.len(), 0),
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn constants() {
            match setup() {
                Ok(file) => assert_eq!(file.header.constants.len(), 0),
                Err(_) => panic!("File could not be read"),
            }
        }


        #[test]
        fn devices() {
            match setup() {
                Ok(file) => {
                    let device = CTIDevice{
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
                        ].iter().map(|&s| String::from(s)).collect(),
                    };
                    let devices = CTIDevices{
                        devices: vec![device],
                    };

                    assert_eq!(file.header.devices, devices);
                },
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn independent_variable() {
            match setup() {
                Ok(file) => {
                    let expected: Vec<f64> = vec![
                        1000000000.,
                        2000000000.,
                        2500000000.,
                        3000000000.,
                    ];

                    assert_eq!(file.header.independent_variable.name, Some(String::from("FREQ")));
                    assert_eq!(file.header.independent_variable.format, Some(String::from("MAG")));
                
                    assert_array_relative_eq!(expected, file.header.independent_variable.data);
                },
                Err(_) => panic!("File could not be read"),
            }
        }

        #[test]
        fn data() {
            match setup() {
                Ok(file) => {
                    let real0: Vec<f64> = vec![1.12134E-3, 4.23145E-3, -0.56815E-3, -1.85942E-3];
                    let imag0: Vec<f64> = vec![1.73103E-3, -5.36775E-3, 5.32650E-3, -4.07981E-3];

                    let real1: Vec<f64> = vec![2.03895E-2, -4.21371E-2, 0.21038E-2, 1.20315E-2];
                    let imag1: Vec<f64> = vec![-0.82674E-2, -0.24871E-2, -3.06778E-2, 5.99861E-2];

                    let real2: Vec<f64> = vec![4.45404E-1, 8.34777E-1, -7.09137E-1, 4.84252E-1];
                    let imag2: Vec<f64> = vec![4.31518E-1, -1.33056E-1, 5.58410E-1, -8.07098E-1];

                    assert_eq!(file.data.len(), 3);
                    assert_eq!(file.data[0].name, Some(String::from("E[1]")));
                    assert_eq!(file.data[0].format, Some(String::from("RI")));
                    assert_array_relative_eq!(real0, file.data[0].real);
                    assert_array_relative_eq!(imag0, file.data[0].imag);

                    assert_eq!(file.data[1].name, Some(String::from("E[2]")));
                    assert_eq!(file.data[1].format, Some(String::from("RI")));
                    assert_array_relative_eq!(real1, file.data[1].real);
                    assert_array_relative_eq!(imag1, file.data[1].imag);

                    assert_eq!(file.data[2].name, Some(String::from("E[3]")));
                    assert_eq!(file.data[2].format, Some(String::from("RI")));
                    assert_array_relative_eq!(real2, file.data[2].real);
                    assert_array_relative_eq!(imag2, file.data[2].imag);
                },
                Err(_) => panic!("File could not be read"),
            }
        }
    }
}
