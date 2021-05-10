use criterion::criterion_main;

pub mod read;
pub mod write;

criterion_main!(write::write, read::read,);
