use citi;
use criterion::{black_box, criterion_group, Criterion};
use num_complex::Complex;
use rand::Rng;
use std::fs::File;
use tempfile::tempdir;

fn create_record(n: usize) -> citi::Record {
    let mut rng = rand::thread_rng();

    let mut record = citi::Record::default();
    record.header.constants.push(citi::Constant {
        name: String::from("Const Name"),
        value: String::from("Value"),
    });
    record.header.independent_variable = citi::Var {
        name: String::from("Var Name"),
        format: String::from("Format"),
        data: vec![1.],
    };
    record.header.devices.push(citi::Device {
        name: String::from("Name A"),
        entries: vec![String::from("entry 1"), String::from("entry 2")],
    });
    record.header.comments.push(String::from("A Comment"));
    record.header.name = String::from("Name");
    record.header.version = String::from("A.01.00");
    record.data.push(citi::DataArray {
        name: String::from("Data Name A"),
        format: String::from("Format A"),
        samples: (0..n)
            .map(|_| Complex {
                re: rng.gen_range(-100.0..100.0),
                im: rng.gen_range(-100.0..100.0),
            })
            .collect(),
    });

    record
}

fn write_record<W: std::io::Write>(record: &citi::Record, writer: &mut W) {
    record.to_writer(writer).unwrap();
}

fn write_benchmark(c: &mut Criterion) {
    c.bench_function("write    10 samples", |b| {
        b.iter(|| {
            write_record(
                black_box(&create_record(10)),
                black_box(&mut File::create(tempdir().unwrap().path().join("file.cti")).unwrap()),
            )
        })
    });
    c.bench_function("write   100 samples", |b| {
        b.iter(|| {
            write_record(
                black_box(&create_record(100)),
                black_box(&mut File::create(tempdir().unwrap().path().join("file.cti")).unwrap()),
            )
        })
    });
    c.bench_function("write  1000 samples", |b| {
        b.iter(|| {
            write_record(
                black_box(&create_record(1000)),
                black_box(&mut File::create(tempdir().unwrap().path().join("file.cti")).unwrap()),
            )
        })
    });
    c.bench_function("write 10000 samples", |b| {
        b.iter(|| {
            write_record(
                black_box(&create_record(10000)),
                black_box(&mut File::create(tempdir().unwrap().path().join("file.cti")).unwrap()),
            )
        })
    });
}

criterion_group!(write, write_benchmark);
