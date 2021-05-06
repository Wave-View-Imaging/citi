#include <cstring>
#include <iostream>
#include <future>
#include <filesystem>

#include <catch2/catch.hpp>
#include <citi/citi.hpp>

namespace fs = std::filesystem;
using namespace citi;

SCENARIO("Writing a record to the file system correctly writes the data to the file", "[Record]") {
    GIVEN("a record read from a file with a modified name") {
        const auto citi_file_path = fs::current_path() / "tests" / "regression_files" / "data_file.cti";
        Record record { citi_file_path };
        
        const std::string record_name { "record_name" };
        record.set_name(record_name);

        WHEN("the record is written to a file") {
            const auto citi_write_file_path = fs::current_path() / "tests" / "temp_test_file.cti";
            record.write_to_file(citi_write_file_path);

            THEN("the file is written to the file system") {
                REQUIRE(fs::exists(citi_write_file_path));

                WHEN("the file is read back again") {
                    Record record_from_file { citi_write_file_path };

                    THEN("the correct values are retrieved") {
                        const auto version = record.version();
                        const auto version_read = record_from_file.version();
                        REQUIRE(version == version_read);

                        const auto name = record.name();
                        const auto name_read = record_from_file.name();
                        REQUIRE(name == name_read);
                    }

                }

            }

            fs::remove(citi_write_file_path);
        }

        WHEN("the record is written to a file in an async manner") {
            const auto citi_write_file_path1 = fs::current_path() / "tests" / "temp_test_file_acync1.cti";
            std::future<void> f1 = std::async(std::launch::async, [&]{
                record.write_to_file(citi_write_file_path1);
            });

            const auto citi_write_file_path2 = fs::current_path() / "tests" / "temp_test_file_acync2.cti";
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
        }
        
    }
}



