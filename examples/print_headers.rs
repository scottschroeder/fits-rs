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

    match fits_rs::parser::parse(&buffer) {
        Ok(fits) => {
            for header_block in &fits.headers {
                println!("{}", header_block)
            }
        }
        Err(e) => match e {
            nom::Err::Incomplete(_) => {
                eprintln!("fits file appeared incomplete: {}", e)
            }
            nom::Err::Error(e) => display_nom_error(e),
            nom::Err::Failure(e) => display_nom_error(e),
        },
    }
}

fn display_nom_error(e: nom::error::Error<&[u8]>) {
    let s = String::from_utf8_lossy(e.input);
    eprintln!("unable to parse header due to '{:?}': {:?}", e.code, s);
}
