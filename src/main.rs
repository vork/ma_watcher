extern crate tue_raw;

use tue_raw::tue_raw_img::Image;
use std::env;

pub fn main() {
    let args: Vec<_> = env::args().collect();

    println!("{:?}", args);
    if args.len() > 2 {
        panic!("Only one argument is allowed!");
    }

    Image::read_img(&args[1]);
}