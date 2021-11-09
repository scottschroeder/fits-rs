use fits_rs::parser::stream_parser::HeaderParser;
use std::{env, fs::File, io::Read};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let mut f = File::open(filename).expect("file not found");
    let mut buffer: Vec<u8> = vec![];
    let _ = f.read_to_end(&mut buffer);

    let mut start = 0;
    loop {
        let input = &buffer.as_slice()[start..];
        if input.is_empty() {
            break;
        }
        let mut parser = HeaderParser::new(start);
        parser.parse_some(input).unwrap();
        let header = parser.into_header();
        println!("{}", header);
        start = header.next_header();
    }
}
