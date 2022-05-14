use lazy_static::lazy_static;
use std::fs::File;
use std::io::Write;
use std::sync::Mutex;

// this will be made better in the future
pub fn log(input: String) {
    let mut f = File::create("vst_out.txt").unwrap();
    write!(f, "{}\n", input).unwrap();
}