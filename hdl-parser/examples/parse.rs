use hdl_parser::{parse_hdl, parse_hdl2};
use std::{env, error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let file = match env::args().nth(1) {
        Some(f) => f,
        None => panic!("expected HDL file"),
    };

    let hdl_src = fs::read_to_string(file).expect("could not open hdl file");
    let hdl = parse_hdl2(&hdl_src).unwrap();
    println!("{hdl:#?}");

    Ok(())
}
