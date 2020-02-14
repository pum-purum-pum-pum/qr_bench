use crossbeam_utils::thread;
use image;
// another library to try for this task
use quirc;

use image::{imageops::FilterType, ImageFormat, Luma};
use qrcode::{types::EcLevel, QrCode};
use quirc::QrCoder;
use std::fs::{self, File};
use std::io::Read;
use std::str::from_utf8;
use std::time::Instant;

fn main() {
    let data = &b"lxsdsds-1:david:cowman:david@gojump-america.com:OS:i:0001110"[..];
    let code = QrCode::with_error_correction_level(data, EcLevel::L).unwrap();
    let image = code.render::<Luma<u8>>().build();
    image.save("qr_generated.png").unwrap();

    let start = Instant::now();
    thread::scope(|s| {
        for entry in fs::read_dir("test").unwrap() {
            let path = entry.unwrap().path();
            let mut file = File::open(path.clone()).unwrap();
            let mut vec = Vec::new();
            file.read_to_end(&mut vec).unwrap();
            let image = image::load_from_memory(&vec)
                .unwrap()
                .resize(800, 600, FilterType::Nearest)
                .to_luma();
            let width = image.width();
            let height = image.height();
            let _handle = s.spawn(move |_| {
                let mut quirc = QrCoder::new().unwrap();
                let codes = quirc.codes(&image, width, height).unwrap();
                println!("{:?}", path);
                for code in codes {
                    match code {
                        Ok(code) => {
                            println!("{:?}", code);
                            println!("{:?}", from_utf8(&code.payload))
                        }
                        Err(err) => println!("Error: {:?}", err),
                    }
                }
            });
            dbg!(Instant::now() - start);
        }
    })
    .unwrap()
}
