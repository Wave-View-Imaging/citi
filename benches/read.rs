use citi;
use criterion::{black_box, criterion_group, Criterion};
use std::fs::File;
use std::path::PathBuf;

fn read_record(filename: &str) {
    let mut path_buf = base_directory();
    path_buf.push(filename);
    let mut reader = File::open(path_buf).unwrap();

    citi::Record::from_reader(&mut reader).unwrap();
}

fn base_directory() -> PathBuf {
    let mut path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path_buf.push("tests");
    path_buf.push("regression_files");
    path_buf
}

macro_rules! read_benchmark {
    ($name: ident, $filename: literal) => {
        fn $name(c: &mut Criterion) {
            c.bench_function($filename, |b| b.iter(|| read_record(black_box($filename))));
        }
    };
}

read_benchmark!(data_file, "data_file.cti");
read_benchmark!(display_memory, "display_memory.cti");
read_benchmark!(list_cal_set, "list_cal_set.cti");
read_benchmark!(wvi_file, "wvi_file.cti");

criterion_group!(read, data_file, display_memory, list_cal_set, wvi_file,);
