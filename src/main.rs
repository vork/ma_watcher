extern crate tue_raw;
extern crate pshbullet_client;
extern crate notify;

use std::env;

use notify::{RecommendedWatcher, Watcher, DebouncedEvent, RecursiveMode};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use pshbullet_client::*;
use pshbullet_client::push::*;

use tue_raw::tue_raw_img::Image;

pub fn main() {
    let args: Vec<_> = env::args().collect();

    println!("{:?}", args);
    if args.len() > 3 {
        panic!("Only two arguments is allowed!");
    }

    //Arguments are: 
    //1: Pushbullet API token
    //2: Folder to watch
    
    let mut path_arg = 1;

    let target = Target::Broadcast;
    let mut client: Option<PushbulletClient> = None;
    if args.len() > 2 {
        let img_num = 1;
        let note_request = Request::Note {
            title: "Captured empty image!",
            body: &format!("Faulty image: {}", img_num)
        };

        client = Some(PushbulletClient::new(String::from(args[2].clone())));
        path_arg = 2;
    }

    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2)).unwrap();

    watcher.watch(&args[path_arg], RecursiveMode::Recursive).unwrap(); // maybe switch to NonRecursive

    loop {
        let path = match rx.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::Write(path) => Some(path),
                    _ => None,
                }
            },
            Err(_) => None,
        };

        if let Some(p) = path {
            
        }
    }

    println!("Done");

    /*let mut img = Image::read_img(&args[1]);

    println!("Image read");

    img.set_clamp_percentage(0.0, 0.0125);

    img.save_as_png("../hdr.png");

    println!("Done");*/
}