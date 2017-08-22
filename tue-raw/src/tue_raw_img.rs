use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::path::Path;
use std::io::Cursor;
use std::str;
use std::str::FromStr;

use nom::{IResult,digit};
use image;
use image::{ImageBuffer, Rgb, Luma, RgbImage, GrayImage, Pixel};
use byteorder::{ReadBytesExt, WriteBytesExt, BigEndian, LittleEndian};

type RawImage = ImageBuffer<Rgb<f32>, Vec<f32>>;
pub struct Image {
    img: RawImage,
    header: Header,
    min_max: (f32, f32),
    visible_min_max: (f32, f32)
}

#[inline]
fn bytes_to_str(bytes: &[u8]) -> &str {
  unsafe { str::from_utf8_unchecked(bytes) }
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

fn calculate_min_max(img: &RawImage) -> (f32, f32) {
    let mut min = 1f32;
    let mut max = 0f32;

    for p in img.pixels() {
        for c in 0..3 {
            if min > p[c] {
                min = p[c];
            }
            if max < p[c] {
                max = p[c];
            }
        }
    }

    (min, max)
}

impl Image {
    pub fn read_img(path: &str) -> Self {
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

        let mut data: Vec<f32> = Vec::with_capacity((header.size.0 * header.size.1 * header.buffer_channels as u32) as usize);

        for y in 0..header.size.1 {
            for x in 0..header.size.0 {
                let mut buffer = match header.buffer_channels {
                    3 => vec![0u8; 12].into_boxed_slice(),
                    1 => vec![0u8; 4].into_boxed_slice(),
                    _ => panic!("Only 1 or 3 channels is supported!"),
                };
                reader.read_exact(&mut buffer);

                for c in 0..header.buffer_channels {
                    data.push(Cursor::new(vec![buffer[c as usize], buffer[(c + 1) as usize], buffer[(c + 2) as usize], buffer[(c + 3) as usize]]).read_f32::<BigEndian>().unwrap());
                }
            }
        }

        let img = ImageBuffer::from_raw(header.size.0, header.size.1, data).unwrap();
        let min_max = calculate_min_max(&img);
        Image{ img: img, header: header, min_max: min_max, visible_min_max: min_max }
    }

    pub fn set_clamp_percentage(&mut self, min_perc: f32, max_perc: f32) {
        if min_perc < 0.0f32 || min_perc > 1.0f32 ||
                max_perc < 0.0f32 || min_perc > 1.0f32 ||
                min_perc > max_perc {
            panic!{"Percentages must be between 0.0 and 1.0 and min can't be larger than max"};
        }
        let range = self.min_max.1 - self.min_max.0;
        let n_min = range * min_perc + self.min_max.0;
        let n_max = range * max_perc + self.min_max.0;

        self.visible_min_max = (n_min, n_max);
    }

    pub fn save_as_png(&self, out: &str) {
        let mut imgbufRgb = RgbImage::new(self.img.width(), self.img.height());
        let mut imgbufLuma = GrayImage::new(self.img.width(), self.img.height());

        for cp in self.img.enumerate_pixels() {
            let mut p = cp.2.clone();
            p.apply(|v| (v - self.visible_min_max.0) / (self.visible_min_max.1 - self.visible_min_max.0) * 255f32);
            p.apply(|v| v.max(0.0f32).min(255.0f32)); //Clamp
            match self.header.buffer_channels {
                3 => imgbufRgb.put_pixel(cp.0, cp.1, image::Rgb{ data: [p[0] as u8, p[1] as u8, p[2] as u8] }),
                1 => imgbufLuma.put_pixel(cp.0, cp.1,image::Luma{ data: [p[0] as u8] }),
                _ => panic!("Only 3 or channels is supported!"),
            };
        }

        let ref mut fout = File::create(&Path::new(out)).unwrap();

        match self.header.buffer_channels {
                3 => image::ImageRgb8(imgbufRgb).save(fout, image::PNG),
                1 => image::ImageLuma8(imgbufLuma).save(fout, image::PNG),
                _ => panic!("Only 3 or channels is supported!"),
        };
    }

    pub fn get_min_max(&self) -> (f32, f32) {
        self.min_max
    }
}