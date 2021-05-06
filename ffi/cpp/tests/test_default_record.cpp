#include <citi/citi.hpp>
#include <catch2/catch.hpp>

#include <cstring>

using namespace citi;

SCENARIO("Creating a default record produces a valid empty Record type.", "[Record]") {
    GIVEN("a default record") {
        
        Record record;

        WHEN("the default version is checked") {
            const auto version = record.version();

            THEN("the default version is returned") {
                REQUIRE(version == "A.01.00");
            }
        }

        WHEN("the default name is checked") {
            const auto name = record.name();

            THEN("an empty string is returned") {
                REQUIRE(name == "");
            }
        }
    }
}


