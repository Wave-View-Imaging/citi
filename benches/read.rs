use citi;
use criterion::{black_box, criterion_group, Criterion};
use std::path::{Path,PathBuf};

fn read_record<P: AsRef<Path>>(path: &P) {
    citi::Record::read(path).unwrap();
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
            let mut path_buf = base_directory();
            path_buf.push($filename);    
        
            c.bench_function($filename, |b| b.iter(|| read_record(black_box(&path_buf))));
        }
    };
}

read_benchmark!(data_file, "data_file.cti");
read_benchmark!(display_memory, "display_memory.cti");
read_benchmark!(list_cal_set, "list_cal_set.cti");
read_benchmark!(wvi_file, "wvi_file.cti");

criterion_group!(read,
    data_file,
    display_memory,
    list_cal_set,
    wvi_file,
);
