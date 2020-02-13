use bardecoder;
use crossbeam_utils::thread;
use image::imageops::FilterType;
use std::time::Instant;
const MSG_NUM: usize = 120;

fn main() {
    // Use default decoder
    let now = Instant::now();
    thread::scope(|s| {
        let mut threads = vec![];
        for _ in 0..MSG_NUM {
            let img = image::open("test.jpg").unwrap();
            // images.push(img);
            let handle = s.spawn(move |_| {
                let decoder = bardecoder::default_decoder();
                let img = img.resize(800, 600, FilterType::Nearest);
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
