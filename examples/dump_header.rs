extern crate fits_rs;
extern crate nom;

use fits_rs::parser::parse_header;
use std::{env, fs::File, io::Read};

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    let mut f = File::open(filename).expect("file not found");
    let mut buffer: Vec<u8> = vec![];
    let _ = f.read_to_end(&mut buffer);

    // let result = parse_header(&buffer[..20 * 2880]);
    let result = parse_header(&buffer);

    match result {
        Ok((i, keywords)) => {
            println!("next 50 bytes: {:?}", &i[..50]);
            for kw in keywords {
                println!("{}", kw);
            }
        }

        // IResult::Done(_, trappist1) => {
        //     for record in trappist1 {
        //         println!("{}", record);
        //     }
        // },
        Err(e) => panic!("Whoops, something went wrong: {}", e),
    }
}
