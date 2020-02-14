use crossbeam_utils::thread;
use image;
// another library to try for this task
use quirc;

use std::fs::{self, File};
use std::io::Read;
use std::time::Instant;
use std::str::from_utf8;
use quirc::QrCoder;
use image::imageops::FilterType;

fn main() {
    let start = Instant::now();
    thread::scope(|s| {
        for entry in fs::read_dir("images").unwrap() {
            let mut file = File::open(entry.unwrap().path()).unwrap();
            let mut vec = Vec::new();
            file.read_to_end(&mut vec).unwrap();
            let image = image::load_from_memory(&vec).unwrap().resize(800, 600, FilterType::Nearest).to_luma();
            let width  = image.width();
            let height = image.height();        
            let _handle = s.spawn(move |_| {
                let mut quirc = QrCoder::new().unwrap();
                let codes  = quirc.codes(&image, width, height).unwrap();
                for code in codes {
                    match code {
                        Ok(code) => {
                            println!("{:?}", code);
                            println!("{:?}", from_utf8(&code.payload))
                        },
                        Err(err) => println!("Error: {:?}", err),
                    }
                }
            });
            dbg!(Instant::now() - start);
        }
    })
    .unwrap()
}