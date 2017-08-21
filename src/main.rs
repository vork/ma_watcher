extern crate tue_raw;

use tue_raw::tue_raw_img::Image;
use std::env;

pub fn main() {
    let args: Vec<_> = env::args().collect();

    println!("{:?}", args);
    if args.len() > 2 {
        panic!("Only one argument is allowed!");
    }

    let mut img = Image::read_img(&args[1]);

    println!("Image read");

    img.set_clamp_percentage(0.0, 0.05);

    img.save_as_png("../hdr.png");

    println!("Done");
}