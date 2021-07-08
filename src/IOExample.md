## IO Example

Read file:
```no_run
use citi::Record;
use std::fs::File;

let mut file = File::open("file.cti").unwrap();
let record = Record::from_reader(&mut file);
```

Write file:
```no_run
use citi::Record;
use std::fs::File;

let record = Record::default();
let mut file = File::create("file.cti").unwrap();
record.to_writer(&mut file);
```
