use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::str;
use std::str::FromStr;
use nom::{IResult,digit};

use std::str::{from_utf8_unchecked};

pub struct Image {
    base_file_name: String,
    size: (u32, u32),
    data: Vec<[f32; 3]>
}

#[inline]
fn bytes_to_str(bytes: &[u8]) -> &str {
  unsafe { from_utf8_unchecked(bytes) }
}

#[inline]
fn bytes_to_string(bytes: &[u8]) -> String {
  bytes_to_str(bytes).to_owned()
}

#[derive(Debug)]
struct Header {
    file_name: String,
    dim: u8,
    size: (u32, u32),
    buffer_channels: u8,
    primtype: String,
    buffer_type: String
}

named!(header_parser<Header>, 
    do_parse!(
        ws!(tag!("IBRraw.xdr")) >>
        file_name: map_res!(
            delimited!( tag!("@@FileBaseName = "), is_not!("\n"), tag!("\n") ),
            str::from_utf8     
        ) >>
        ws!(tag!("@@FileID = IBRraw")) >>
        dim: map_res!(
                map_res!(
                    delimited!(tag!("@@ImageDim = "), is_not!("\n"), tag!("\n")),
                    str::from_utf8
                ),
                FromStr::from_str
        ) >>
        tag!("@@ImageSize =") >>
        width: map_res!(
                map_res!(
                    delimited!(tag!(" "), is_not!(" "), tag!(" ")),
                    str::from_utf8
                ),
                FromStr::from_str
        ) >>
        height: map_res!(
                map_res!(
                    take_until_and_consume_s!("\n"),
                    str::from_utf8
                ),
                FromStr::from_str
        ) >>
        channels: map_res!(
                map_res!(
                    delimited!(tag!("@@buffer-channels-0 = "), is_not!("\n"), tag!("\n")),
                    str::from_utf8
                ),
                FromStr::from_str
        ) >>
        primtype: map_res!(
            delimited!( tag!("@@buffer-primtype-0 = "), is_not!("\n"), tag!("\n") ),
            str::from_utf8     
        ) >>
        buffer_type: map_res!(
            delimited!( tag!("@@buffer-type-0 = "), is_not!("\n"), tag!("\n") ),
            str::from_utf8     
        ) >>
        ws!(tag!("---end-of-header---")) >>
        (Header {
            file_name: file_name.to_owned(),
            dim: dim,
            size: (width, height),
            buffer_channels: channels,
            primtype: primtype.to_owned(),
            buffer_type: buffer_type.to_owned()
        } )
    )
);


impl Image {
    pub fn read_img(path: &str) {
        let mut f = File::open(path).unwrap();
        let mut reader = BufReader::new(f);

        let mut line = String::new();
        let mut len = reader.read_line(&mut line).unwrap(); //Intro
        len = reader.read_line(&mut line).unwrap(); //Filename
        len = reader.read_line(&mut line).unwrap(); //FileID
        len = reader.read_line(&mut line).unwrap(); //Dimensions
        len = reader.read_line(&mut line).unwrap(); //Size
        len = reader.read_line(&mut line).unwrap(); //Buffer Channels
        len = reader.read_line(&mut line).unwrap(); //PrimType
        len = reader.read_line(&mut line).unwrap(); //Buffer Type
        len = reader.read_line(&mut line).unwrap(); //END

        let (_, header) = header_parser(line.as_bytes()).unwrap();

        println!("{:?}", header);

        println!("Done");

        
    }
}