use bardecoder;
use crossbeam_utils::thread;
use image::imageops::FilterType;
use std::time::Instant;
use std::fs::{self, DirEntry};
use std::path::Path;
use bardecoder::prepare::BlockedMean;
const MSG_NUM: usize = 120;

fn main() {
    // Use default decoder
    let now = Instant::now();
    thread::scope(|s| {
        let mut threads = vec![];
        // for _ in 0..MSG_NUM {
        for entry in fs::read_dir("images2").unwrap() {
        	// let filename = entry.unwrap().file_name();
        	// dbg!(&filename);

            // let mut img = image::open("IMG_2186.jpg").unwrap();
            // img = img.grayscale();
            let img = image::open(entry.unwrap().path()).unwrap();
            // images.push(img);
            let handle = s.spawn(move |_| {
                let mut decoder = bardecoder::default_decoder();
                // decoder.prepare(Box::new(BlockedMean::new(7, 9)));
                let img = img.resize(800, 600, FilterType::Nearest);
	            // img.save(Path::new("resized.jpg"));
                let results = decoder.decode(img);
                println!("{:?}", results);
            });
            threads.push(handle);
            println!("{:?}", Instant::now() - now);
        }
        for thread in threads.drain(..) {
            thread.join().unwrap();
        }
    })
    .unwrap();
}
