include(FetchContent)

# Import Catch2 testing library
FetchContent_Declare(
  Catch2
  GIT_REPOSITORY https://github.com/catchorg/Catch2.git
  GIT_TAG v2.13.4
  )

FetchContent_MakeAvailable(Catch2)

set(TEST_EXEC "test_exec")

add_executable(
    ${TEST_EXEC}
    test_main.cpp
    test_default_record.cpp
    test_read_data_record.cpp
    test_write_record.cpp
)

target_compile_features(${TEST_EXEC} PRIVATE cxx_std_17)

target_link_libraries(
    ${TEST_EXEC}
    PRIVATE
    ${PROJECT_NAME}::${PROJECT_NAME}
)
target_link_libraries(${TEST_EXEC} PRIVATE Catch2::Catch2)

# Register tests
list(APPEND CMAKE_MODULE_PATH ${catch2_SOURCE_DIR}/contrib)
include(Catch)
catch_discover_tests(${TEST_EXEC})
