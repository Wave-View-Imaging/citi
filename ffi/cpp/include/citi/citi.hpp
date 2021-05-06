#ifndef CITI_H
#define CITI_H

#include <filesystem>
#include <string>
#include <vector>
#include <complex>

namespace fs = std::filesystem;

/// C++ interface to the citi utilities from Rust
///
/// The rust ffi exposes C functions to access and manipulate rust types;
/// this C++ library acts as a thin wrapper for the C interface. Note however,
/// that due how data ownership must be maintained, there are some inefficiencies
/// due to extra required data copies.
///
/// Note also that ErrorCodes must be maintained and kept the same on both the Rust and C++ side
/// 
namespace citi {

    typedef void RustRecord;
    class Record {
        public:

        /// The rust ffi uses integer error codes to convey
        /// error variants; this enum is an exact copy of the
        /// same type from the Rust side and as such must be
        /// maintained together.
        enum class ErrorCode {
            NoError = 0,
            UnknownError = -1,

            NullArgument = -3,

            //CStr::from_ptr
            InvalidUTF8String = -4,

            // File::open, File::create
            FileNotFound = -5,
            FilePermissionDenied = -6,
            FileConnectionRefused = -7,
            FileConnectionReset = -8,
            FileConnectionAborted = -9,
            FileNotConnected = -10,
            FileAddrInUse = -11,
            FileAddrNotAvailable = -12,
            FileBrokenPipe = -13,
            FileAlreadyExists = -14,
            FileWouldBlock = -15,
            FileInvalidInput = -16,
            FileInvalidData = -17,
            FileTimedOut = -18,
            FileWriteZero = -19,
            FileInterrupted = -20,
            FileUnexpectedEof = -21,

            // Record::from_reader
            // Record parse errors
            RecordParseErrorBadKeyword = -22,
            RecordParseErrorBadRegex = -23,
            RecordParseErrorNumber = -24,

            // Record read errors
            RecordReadErrorDataArrayOverIndex = -25,
            RecordReadErrorIndependentVariableDefinedTwice = -26,
            RecordReadErrorSingleUseKeywordDefinedTwice = -27,
            RecordReadErrorOutOfOrderKeyword = -28,
            RecordReadErrorLineError = -29,
            RecordReadErrorIO = -30,
            RecordReadErrorNoVersion = -31,
            RecordReadErrorNoName = -32,
            RecordReadErrorNoIndependentVariable = -33,
            RecordReadErrorNoData = -34,
            RecordReadErrorVarAndDataDifferentLengths = -35,

            // Record write errors
            RecordWriteErrorNoVersion = -36,
            RecordWriteErrorNoName = -37,
            RecordWriteErrorNoDataName = -38,
            RecordWriteErrorNoDataFormat = -39,
            RecordWriteErrorWrittingError = -40,

            // CString::new
            NullByte = -41,

            IndexOutOfBounds = -42
        };

        class RuntimeException : public std::runtime_error {
            public:
            explicit RuntimeException(std::string&& description, ErrorCode error_code) noexcept;
            const char* what () const noexcept override;

            private:
            std::string message;
            ErrorCode error_code;
        };

        struct Device {
            std::string name;
            std::vector<std::string> entries;
        };

        struct IndependentVariable {
            std::string name;
            std::string format;
            std::vector<double> values;
        };

        struct DataArray {
            std::string name;
            std::string format;
            std::vector<std::complex<double>> samples;
        };

        static ErrorCode error_code_from_int(int error_code_int);

        explicit Record();  
        explicit Record(const fs::path& filename);
        ~Record();

        std::string version();
        void set_version(const std::string& version);
        std::string name();
        void set_name(const std::string& name);
        std::vector<std::string> comments();
        void append_comment(const std::string& comment);
        std::vector<Device> devices();
        void append_device(const Device& device);
        IndependentVariable independent_variable();
        void set_independent_variable(const IndependentVariable& var);
        std::vector<DataArray> data();
        void append_data_array(const DataArray& data_arr);
        void write_to_file(const fs::path& filename);

        private:
        RustRecord* rust_record;
    };
}

#endif
