use std::{
    fs::{self, File},
    io::Write,
    time::SystemTime,
};

use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;

static mut LOG_FILE: Lazy<File> = Lazy::new(|| {
    fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open("C:\\Users\\cn\\file.txt")
        .unwrap()
});

pub unsafe fn log(data: &[u8]) {
    let now = SystemTime::now();
    let datetime = DateTime::<Utc>::from(now);
    let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S.%f").to_string();

    LOG_FILE.write_all(timestamp_str.as_bytes()).unwrap();

    LOG_FILE.write_all(data).unwrap();
    LOG_FILE.write_all(b"\n").unwrap();
}
