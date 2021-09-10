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

            NullArgument = -2,

            //CStr::from_ptr
            InvalidUTF8String = -3,

            // File::open, File::create
            FileNotFound = -4,
            FilePermissionDenied = -5,
            FileConnectionRefused = -6,
            FileConnectionReset = -7,
            FileConnectionAborted = -8,
            FileNotConnected = -9,
            FileAddrInUse = -10,
            FileAddrNotAvailable = -11,
            FileBrokenPipe = -12,
            FileAlreadyExists = -13,
            FileWouldBlock = -14,
            FileInvalidInput = -15,
            FileInvalidData = -16,
            FileTimedOut = -17,
            FileWriteZero = -18,
            FileInterrupted = -19,
            FileUnexpectedEof = -20,

            // Record::from_reader
            // Record parse errors
            RecordParseErrorBadKeyword = -21,
            RecordParseErrorBadRegex = -22,
            RecordParseErrorNumber = -23,

            // Record read errors
            RecordReadErrorDataArrayOverIndex = -24,
            RecordReadErrorIndependentVariableDefinedTwice = -25,
            RecordReadErrorSingleUseKeywordDefinedTwice = -26,
            RecordReadErrorOutOfOrderKeyword = -27,
            RecordReadErrorLineError = -28,
            RecordReadErrorIO = -29,
            RecordReadErrorNoVersion = -30,
            RecordReadErrorNoName = -31,
            RecordReadErrorNoIndependentVariable = -32,
            RecordReadErrorNoData = -33,
            RecordReadErrorVarAndDataDifferentLengths = -34,

            // Record write errors
            RecordWriteErrorNoVersion = -35,
            RecordWriteErrorNoName = -36,
            RecordWriteErrorNoDataName = -37,
            RecordWriteErrorNoDataFormat = -38,
            RecordWriteErrorWrittingError = -39,

            // CString::new
            NullByte = -40,

            IndexOutOfBounds = -41
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
