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
        client = Some(PushbulletClient::new(String::from(args[1].clone())));
        path_arg = 2;
    }

    let (tx, rx) = channel();

    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2)).unwrap();

    watcher.watch(&args[path_arg], RecursiveMode::Recursive).unwrap(); // maybe switch to NonRecursive

    loop {
        let path = match rx.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::Create(path) | DebouncedEvent::Write(path) => Some(path),
                    _ => None,
                }
            },
            Err(_) => None,
        };

        if let Some(p) = path {
            if let Some(ext) = p.extension() {
                if ext == "raw" {
                    if let Some(path) = p.to_str() {
                        if let Ok(img) = Image::read_img(path) {
                            let (_, max) = img.get_min_max();
                            if max == 0f32 { //No data in image?!
                                let note_request = Request::Note {
                                    title: "Captured dark image!",
                                    body: &format!("Faulty image: {}", path)
                                };
                                match client {
                                    Some(ref cli) => {
                                        cli.create_push(&target, note_request).unwrap();
                                    },
                                    None => println!("{:?}", note_request),
                                }
                                
                            }
                        } else { //Image couldn't be parsed?!
                            let note_request = Request::Note {
                                title: "Image couldn't be read!",
                                body: &format!("Faulty image: {}", path)
                            };
                            match client {
                                Some(ref cli) => {
                                    cli.create_push(&target, note_request).unwrap();
                                },
                                None => println!("{:?}", note_request),
                            }
                        }
                    }
                }
            }   
        }
    }\
}