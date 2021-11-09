extern crate fits_rs;
extern crate nom;

use fits_rs::parser::{
    parse_header,
    stream_parser::{HeaderParser, ParseOutcome},
};
use std::{
    env,
    fs::File,
    io::{Read, Seek, SeekFrom},
};

fn parse_loop<'a>(mut buffer: &'a [u8]) -> (HeaderParser<'a>, nom::IResult<&'a [u8], ()>) {
    let mut h = fits_rs::parser::stream_parser::HeaderParser::new();
    loop {
        match h.parse_record(&buffer) {
            ParseOutcome::Ok(r) => buffer = r,
            ParseOutcome::Complete(c) => return (h, Ok((c, ()))),
            ParseOutcome::Error(e) => return (h, Err(e)),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let mut f = File::open(filename).expect("file not found");
    let mut buffer: Vec<u8> = vec![];
    let x = f.seek(SeekFrom::Start(0x05ec5540));
    println!("x: {:?}", x);
    let _ = f.read_to_end(&mut buffer);

    let mut input = buffer.as_slice();

    let mut h = HeaderParser::new();
    input = h.parse_some(input).unwrap();
    let mut h2 = HeaderParser::new();
    input = h2.parse_some(input).unwrap();

    let phdu = h.into_header();
    let xhdu = h2.into_header();

    println!("{}", phdu);
    println!("DAS: {}", phdu.data_array_size());
    println!("{}", xhdu);
    println!("DAS: {}", xhdu.data_array_size());
}
