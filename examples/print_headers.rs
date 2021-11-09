//! Print out all the headers from a .fits file
//! Usage:
//! cargo run --example print_headers /path/to/my/file.fits

use std::{env, fs::File, io::Read};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let mut f = File::open(filename).expect("file not found");
    let mut buffer: Vec<u8> = vec![];
    let _ = f.read_to_end(&mut buffer);

    let fits = fits_rs::parser::parse(&buffer).unwrap();
    for header_block in &fits.headers {
        println!("{}", header_block)
    }
}
