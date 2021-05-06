cmake_minimum_required(VERSION 3.18)

project(
    CitiCpp
    VERSION 0.2.0
    DESCRIPTION "C++ interface to Rust library for reading and writing CITI files"
    LANGUAGES CXX
)

configure_file(version.h.in version.h)

set(RUST_LIB citi)
set(RUST_ROOT_DIR ${CMAKE_CURRENT_SOURCE_DIR})
set(CPP_ROOT_DIR "${CMAKE_CURRENT_SOURCE_DIR}/ffi/cpp")
set(CPP_SRC_DIR "${CPP_ROOT_DIR}/src")
set(CPP_TESTS_DIR "${CPP_ROOT_DIR}/tests")
set(PROJECT_INCLUDE_DIR "${CPP_ROOT_DIR}/include")

# Only do these if this is the main project, and not if it is included through add_subdirectory
if(CMAKE_PROJECT_NAME STREQUAL PROJECT_NAME)
    # Ensure -std=c++xx instead of -std=g++xx
    set(CMAKE_CXX_EXTENSIONS OFF)

    # Support folders in IDEs
    set_property(GLOBAL PROPERTY USE_FOLDERS ON)

    find_package(Doxygen)
    if(Doxygen_FOUND)
        add_subdirectory(docs)
    else()
        message(STATUS "Doxygen not found, not building docs")
    endif()
endif()

add_subdirectory(${CPP_SRC_DIR})

# Testing only available if this is the main app
# Emergency override MODERN_CMAKE_BUILD_TESTING provided as well
if(CMAKE_PROJECT_NAME STREQUAL PROJECT_NAME OR BUILD_TESTING)
    add_subdirectory(${CPP_TESTS_DIR})
endif()
