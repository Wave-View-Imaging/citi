#include <citi/citi.hpp>

extern "C" {
    #include "citi_c_interface.h"
}

namespace {
    citi::Record::RuntimeException record_runtime_exception(int error_code_int) {
        const auto error_code = citi::Record::error_code_from_int(error_code_int);
        return citi::Record::RuntimeException(
            std::string { get_error_description(error_code_int) },
            error_code);
    }

    template <typename T>
    T* check_ptr(T* ptr) {
        if (!ptr) {
            auto error_code_int = get_last_error_code();
            throw record_runtime_exception(error_code_int);
        }
        return ptr;
    }

    void check_int_error_code(int error_code_int) {
        if (citi::Record::error_code_from_int(error_code_int) != citi::Record::ErrorCode::NoError) {
            throw record_runtime_exception(error_code_int);
        } 
    }
}

namespace citi {

    Record::RuntimeException::RuntimeException(std::string&& description, ErrorCode error_code) noexcept : 
        std::runtime_error(description),
        message(std::move(description)),
        error_code(error_code) {}

    const char* Record::RuntimeException::what() const noexcept {
        return message.c_str();
    }

    /// This function currently doesn't check to see whether the input
    /// integer values are represented in the enum.
    Record::ErrorCode Record::error_code_from_int(int error_code) {
        return static_cast<ErrorCode>(error_code);
    }

    Record::Record() {
        rust_record = record_default();  
    }

    Record::Record(const fs::path& filename) {
        rust_record = record_read(filename.string().c_str());  
        check_ptr(rust_record);
    }

    Record::~Record() {
        auto error_code_int = record_destroy(rust_record);
        check_int_error_code(error_code_int);
    }

    std::string Record::version() {
        auto version = check_ptr(record_get_version(rust_record));
        return std::string { version };
    }

    void Record::set_version(const std::string& version) {
        auto error_code_int = record_set_version(rust_record, version.c_str());
        check_int_error_code(error_code_int);
    }

    std::string Record::name() {
        auto name = check_ptr(record_get_name(rust_record));
        return std::string { name };
    }

    void Record::set_name(const std::string& name) {
        auto error_code_int = record_set_name(rust_record, name.c_str());
        check_int_error_code(error_code_int);
    }

    std::vector<std::string> Record::comments() {
        const auto num_comments = record_get_number_of_comments(rust_record);
        // Throws in case of errors
        if (num_comments < 0) {
            check_int_error_code(num_comments);
        }
        std::vector<std::string> comments;
        comments.reserve(num_comments);

        for (auto i = 0; i < num_comments; ++i) {
            const auto comment = check_ptr(record_get_comment(rust_record, i));
            comments.push_back(std::string { comment });
        }
        
        return comments;
    }

    void Record::append_comment(const std::string& comment) {
        const auto error_code_int = record_append_comment(rust_record, comment.c_str());
        check_int_error_code(error_code_int);
    }

    std::vector<Record::Device> Record::devices() {

        const auto num_devices = record_get_number_of_devices(rust_record);
        // Throws in case of errors
        if (num_devices < 0) {
            check_int_error_code(num_devices);
        }
        std::vector<Record::Device> devices;
        devices.reserve(num_devices);

        for (auto i = 0; i < num_devices; ++i) {
            const auto name = std::string { check_ptr(record_get_device_name(rust_record, i)) };

            const auto num_entries = record_get_device_number_of_entries(rust_record, i );

            // Throws in case of errors
            if (num_entries < 0) {
                check_int_error_code(num_entries);
            }
            std::vector<std::string> entries;
            entries.reserve(num_entries);

            for (auto j = 0; j < num_entries; ++j) {
                const auto entry = std::string { check_ptr(record_get_device_entry(rust_record, i, j)) };
                entries.push_back(std::move(entry));
            }

            devices.push_back({
                std::move(name),
                std::move(entries)
            });
        }

        return devices;
    }

    void Record::append_device(const Device& device) {
        const auto last_device_index = record_get_number_of_devices(rust_record);
        // Throws in case of errors
        if (last_device_index < 0) {
            check_int_error_code(last_device_index);
        }

        const auto device_error_code_int = record_append_device(rust_record, device.name.c_str());
        check_int_error_code(device_error_code_int);

        for (const auto entry : device.entries) {
            const auto entry_error_code_int =
                record_append_entry_to_device(rust_record, last_device_index, entry.c_str());
            check_int_error_code(entry_error_code_int);
        }
    }

    Record::IndependentVariable Record::independent_variable() {
        const std::string name { check_ptr(record_get_independent_variable_name(rust_record)) };
        const std::string format { check_ptr(record_get_independent_variable_format(rust_record)) };
        
        const auto num_vals = record_get_independent_variable_length(rust_record);
        // Throws in case of errors
        if (num_vals < 0) {
            check_int_error_code(num_vals);
        } 
        const auto array = check_ptr(record_get_independent_variable_array(rust_record));

        std::vector<double> values;
        values.reserve(num_vals);
        for (auto i = 0; i < num_vals; ++i) {
            values.push_back(array[i]);
        }
        
        return {
            std::move(name),
            std::move(format),
            std::move(values)
        };
    }

    void Record::set_independent_variable(const IndependentVariable& var) {
        const auto error_code_int = record_set_independent_variable(
            rust_record,
            var.name.c_str(), var.format.c_str(),
            var.values.data(), var.values.size());

        check_int_error_code(error_code_int);
    }
        
    std::vector<Record::DataArray> Record::data() {
        const auto num_data_arrays = record_get_number_of_data_arrays(rust_record);
        // Throws in case of errors
        if (num_data_arrays < 0) {
            check_int_error_code(num_data_arrays);
        }
        
        std::vector<Record::DataArray> data_arrays;
        data_arrays.reserve(num_data_arrays);
        for (auto i = 0; i < num_data_arrays; ++i) {
            const std::string name { check_ptr(record_get_data_array_name(rust_record, i)) };
            const std::string format { check_ptr(record_get_data_array_format(rust_record, i)) };

            const auto data_array_length = record_get_data_array_length(rust_record, i);
            // Throws in case of errors
            if (data_array_length < 0) {
                check_int_error_code(data_array_length);
            }
            
            std::vector<double> reals(data_array_length);
            std::vector<double> imags(data_array_length);
            const auto error_code_int = record_get_data_array(rust_record, i, reals.data(), imags.data());
            check_int_error_code(error_code_int);

            std::vector<std::complex<double>> samples(data_array_length);
            std::transform(reals.begin(), reals.end(), imags.begin(), samples.begin(),
                [](double re, double im) {

                return std::complex<double>(re, im);
            });

            data_arrays.push_back({
                std::move(name),
                std::move(format),
                std::move(samples)
            });
        }

        return data_arrays;
    }

    void Record::append_data_array(const DataArray& data_arr) {
        std::vector<double> reals;
        reals.reserve(data_arr.samples.size());
        std::vector<double> imags;
        imags.reserve(data_arr.samples.size());
        for (auto i = 0; i < data_arr.samples.size(); ++i) {
            reals.push_back(data_arr.samples[i].real());
            imags.push_back(data_arr.samples[i].imag());
        }
        const auto error_code_int = record_append_data_array(
            rust_record, data_arr.name.c_str(), data_arr.format.c_str(),
            reals.data(), imags.data(), imags.size());

        check_int_error_code(error_code_int);
    }

    void Record::write_to_file(const fs::path& filename) {
        const auto error_code_int = record_write(rust_record, filename.string().c_str());  
        check_int_error_code(error_code_int);
    }
}
