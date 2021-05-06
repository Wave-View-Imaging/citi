#include <cstring>
#include <iostream>
#include <future>
#include <filesystem>

#include <catch2/catch.hpp>
#include <citi/citi.hpp>

namespace fs = std::filesystem;
using namespace citi;

SCENARIO("Reading a valid file into a record produces the correct values.", "[Record]") {
    GIVEN("a record read from a file") {
        
        const auto citi_file_path = fs::current_path() / "tests" / "regression_files" / "data_file.cti";
        Record record { citi_file_path };

        WHEN("the version is checked") {
            const auto version = record.version();

            THEN("the default version is returned") {
                REQUIRE(version == "A.01.00");
            }
        }

        WHEN("the name is checked") {
            const auto name = record.name();

            THEN("the correct name is returned") {
                REQUIRE(name == "DATA");
            }
        }

        WHEN("the comments are retrieved") {
            const auto comments = record.comments();

            THEN("there are none") {
                REQUIRE(comments.size() == 0);
            }
        }

        WHEN("a comment is added") {
            const auto test_comment = "this is definitely a comment";
            record.append_comment(test_comment);

            THEN("the same comment can be retrieved") {

                const auto comments = record.comments();
                REQUIRE(comments.size() == 1);
                REQUIRE(comments[0] == test_comment);
            }
        }


        WHEN("the devices are retrieved") {
            const auto devices = record.devices();

            THEN("the correct device is found") {
                const Record::Device comparison_device {
                    "NA",
                    { "VERSION HP8510B.05.00", "REGISTER 1" }
                };
                
                REQUIRE(devices.size() == 1);
                REQUIRE(devices[0].name == comparison_device.name);
                REQUIRE(devices[0].entries == comparison_device.entries);
            }
        }

        WHEN("a device is added") {
            const Record::Device device_to_add {
                "Device Name",
                { "ASDF", "asdf" }
            };
            record.append_device(device_to_add);

            THEN("the same device can be retrieved") {
                const auto devices = record.devices();
                const auto last_device = devices[devices.size() - 1];

                REQUIRE(last_device.name == device_to_add.name);
                REQUIRE(last_device.entries == device_to_add.entries);
            }
        }

        WHEN("the independent variable data is retrieved") {
            const auto ivar = record.independent_variable();

            THEN("the correct variable data is found") {
                const Record::IndependentVariable comparison_ivar {
                    "FREQ", "MAG",
                    {
                        1.00000000e+09, 1.33333333e+09, 1.66666667e+09,
                        2.00000000e+09, 2.33333333e+09, 2.66666667e+09,
                        3.00000000e+09, 3.33333333e+09, 3.66666667e+09, 4.00000000e+09
                    }
                };
                
                REQUIRE(ivar.name == comparison_ivar.name);
                REQUIRE(ivar.format == comparison_ivar.format);
                REQUIRE_THAT(ivar.values, Catch::Approx(comparison_ivar.values));
            }
        }

        WHEN("an independent variable is set") {
            const Record::IndependentVariable new_ivar {
                "FREQ", "PHASE",
                { 0.5, 0.6, 0.7, 0.8, 1.0 }
            };
            record.set_independent_variable(new_ivar);

            THEN("the same indepedent variable data is retrieved") {
                const auto ivar = record.independent_variable();
                
                REQUIRE(ivar.name == new_ivar.name);
                REQUIRE(ivar.format == new_ivar.format);
                REQUIRE_THAT(ivar.values, Catch::Approx(new_ivar.values));
            }
        }

        WHEN("the data arrays are retrieved") {
            const auto data_arrays = record.data();

            THEN("the correct data is found") {
                const Record::DataArray comparison_data_array {
                    "S[1,1]", "RI",
                    {
                        { 0.86303E-1, -8.98651E-1 },
                        { 8.97491E-1, 3.06915E-1 },
                        { -4.96887E-1, 7.87323E-1 },
                        { -5.65338E-1, -7.05291E-1 },
                        { 8.94287E-1, -4.25537E-1 },
                        { 1.77551E-1, 8.96606E-1 },
                        { -9.35028E-1, -1.10504E-1 },
                        { 3.69079E-1, -9.13787E-1 },
                        { 7.80120E-1, 5.37841E-1 },
                        { -7.78350E-1, 5.72082E-1 },
                    }
                };
                
                REQUIRE(data_arrays.size() == 1);

                const auto& first_data_array = data_arrays[0];
                REQUIRE(first_data_array.name == comparison_data_array.name);
                REQUIRE(first_data_array.format == comparison_data_array.format);

                REQUIRE(first_data_array.samples.size() == comparison_data_array.samples.size());
                for (auto i = 0; i < first_data_array.samples.size(); ++i) {
                    REQUIRE(first_data_array.samples[i].real() == comparison_data_array.samples[i].real());
                    REQUIRE(first_data_array.samples[i].imag() == comparison_data_array.samples[i].imag());
                }
            }
        }

        WHEN("a data array is appended") {
            const Record::DataArray new_data_array {
                "S[2, 2]", "RI",
                {
                    { 0.86303E-1, -8.98651E-1 },
                    { 8.97491E-1, 3.06915E-1 },
                    { -4.96887E-1, 7.87323E-1 },
                    { -5.65338E-1, -7.05291E-1 },
                    { 8.94287E-1, -4.25537E-1 },
                }
            };
            record.append_data_array(new_data_array);

            THEN("the appended data array can be retrieved") {
                const auto data_arrays = record.data();

                REQUIRE(data_arrays.size() == 2);

                const auto& second_data_array = data_arrays[1];
                REQUIRE(second_data_array.name == new_data_array.name);
                REQUIRE(second_data_array.format == new_data_array.format);

                REQUIRE(second_data_array.samples.size() == new_data_array.samples.size());
                for (auto i = 0; i < second_data_array.samples.size(); ++i) {
                    REQUIRE(second_data_array.samples[i].real() == new_data_array.samples[i].real());
                    REQUIRE(second_data_array.samples[i].imag() == new_data_array.samples[i].imag());
                }
            }
        }

        /*
        WHEN("the record is written to a file") {
            const auto citi_write_file_path = fs::current_path() / "tests" / "regression_files" / "temp_test_file.cti";
            record.write_to_file(citi_write_file_path);

            THEN("the file is written to the file system") {
                REQUIRE(fs::exists(citi_write_file_path));
            }

            fs::remove(citi_write_file_path);
        }

        WHEN("the record is written to a file in an async manner") {
            const auto citi_write_file_path1 = fs::current_path() / "tests" / "regression_files" / "temp_test_file_acync1.cti";
            std::future<void> f1 = std::async(std::launch::async, [&]{
                record.write_to_file(citi_write_file_path1);
            });

            const auto citi_write_file_path2 = fs::current_path() / "tests" / "regression_files" / "temp_test_file_acync2.cti";
            std::future<void> f2 = std::async(std::launch::async, [&]{
                record.write_to_file(citi_write_file_path2);
            });

            THEN("the files are written to the file system") {
                // Wait for futures to resolve
                f1.get();
                f2.get();

                REQUIRE(fs::exists(citi_write_file_path1));
                REQUIRE(fs::exists(citi_write_file_path2));
            }

            fs::remove(citi_write_file_path1);
            fs::remove(citi_write_file_path2);
        }*/
        
    }
}


