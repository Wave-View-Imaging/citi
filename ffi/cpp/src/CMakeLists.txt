include(FetchContent)

# Import library to manage Rust side
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/AndrewGaspar/corrosion.git
    GIT_TAG origin/master
)
FetchContent_MakeAvailable(Corrosion)
corrosion_import_crate(MANIFEST_PATH "${RUST_ROOT_DIR}/Cargo.toml")

add_library(
    ${PROJECT_NAME}
    STATIC
    citi.cpp
)
add_library(${PROJECT_NAME}::${PROJECT_NAME} ALIAS ${PROJECT_NAME})

target_compile_features(${PROJECT_NAME} PRIVATE cxx_std_17)

target_include_directories(
    ${PROJECT_NAME}
    PUBLIC
    "$<BUILD_INTERFACE:${PROJECT_INCLUDE_DIR}>"
    "$<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>"
)
target_include_directories(${PROJECT_NAME} PRIVATE ./)

# Link to Rust library
target_link_libraries(${PROJECT_NAME} PRIVATE ${RUST_LIB})

# TODO Uncomment and test the following to allow for local installation
# Install the target and create export-set
# include(GNUInstallDirs)
# set(PROJECT_TARGETS "${PROJECT_NAME}Targets")
# install(
#     TARGETS ${PROJECT_NAME}
#     EXPORT ${PROJECT_TARGETS}
#     LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
#     ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
#     RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
#     INCLUDES DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
# )
# 
# # Install header file
# install(
#     FILES 
#     "${PROJECT_INCLUDE_DIR}/citi/citi.hpp"
#     DESTINATION 
#     ${CMAKE_INSTALL_INCLUDEDIR}
# )
# 
# # Generate and install export file
# install(
#     EXPORT ${PROJECT_TARGETS}
#     FILE "${PROJECT_NAME}${PROJECT_TARGETS}.cmake"
#     NAMESPACE "${PROJECT_NAME}::"
#     DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}"
# )
