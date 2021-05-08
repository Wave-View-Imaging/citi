#![feature(test)]

use criterion::criterion_main;

pub mod write;
pub mod read;

criterion_main!(
    write::write,
    read::read,
);