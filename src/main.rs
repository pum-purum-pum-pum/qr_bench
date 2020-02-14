use crossbeam_utils::thread;
use image;
// another library to try for this task
use quirc;

use image::{imageops::{FilterType, blur, unsharpen}, ImageFormat, Luma};
use qrcode::{types::EcLevel, QrCode};
use quirc::{QrCoder, Error, Codes};
use std::sync::atomic::AtomicBool;
use std::fs::{self, File};
use std::io::Read;
use std::str::from_utf8;
use std::time::Instant;
use std::sync::atomic::Ordering;

const FIRST_TAKE: usize = 10;
const BLUR_SIZE: f32 = 4.;

static QR_FOUND: AtomicBool = AtomicBool::new(false);

pub fn extract_code(codes: Codes) {
    for code in codes {
        if let Ok(code) = code {
            println!("{:?}", code);
            println!("{:?}", from_utf8(&code.payload))
        }
    }
}

#[test]
fn order_test() {
    let mut names = vec!["a", "b", "c", "d"];
    assert_eq!(order(&mut names, 1), ["a", "d", "b", "c"]);
}

pub fn order<T>(names: &mut Vec<T>, first_take: usize) -> Vec<T> {
    let mut ordered_filenames: Vec<_> = names.drain(..first_take).collect();
    let last = names.drain((names.len() - first_take)..);
    ordered_filenames.extend(last);
    ordered_filenames.extend(names.drain(..));
    ordered_filenames
}

fn main() {
    if false { // TODO generator if needed
        let data = &b"lxsdsds-1:david:cowman:david@gojump-america.com:OS:i:0001110"[..];
        let code = QrCode::with_error_correction_level(data, EcLevel::L).unwrap();
        let image = code.render::<Luma<u8>>().build();
        image.save("qr_generated.png").unwrap();
    }

    let mut file_names = vec![];
    for entry in fs::read_dir("images").unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {continue};
        file_names.push(path);
    }
    // sort and take the first N the last N and then the middle

    // use std::cmp::Reverse;
    // file_names.sort_by_key(|ref file_name| Reverse(file_name));
    file_names.sort();
    file_names.reverse();
    let ordered_filenames = if file_names.len() > 2 * FIRST_TAKE {
        order(&mut file_names, FIRST_TAKE)
    } else {
        file_names
    };


    let start = Instant::now();
    thread::scope(|s| {
        for path in ordered_filenames.iter() {
            if QR_FOUND.load(Ordering::SeqCst) {
                println!("QR ALREADY FOUND.");
                break;
            }
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
                println!("{:?}", QR_FOUND.load(Ordering::SeqCst));
                if QR_FOUND.load(Ordering::SeqCst) {
                    println!("QR ALREADY FOUND. END OF THE TASK");
                    return;
                }
                let mut quirc = QrCoder::new().unwrap();
                let codes = quirc.codes(&image, width, height).unwrap();
                // println!("");
                // println!("{:?}", path);
                for code in codes {
                    match code {
                        Ok(code) => {
                            QR_FOUND.compare_and_swap(false, true, Ordering::SeqCst);
                            println!("{:?}", from_utf8(&code.payload))
                        }
                        Err(err) => {
                            // // if QuircErrorDataEcc case
                            // let err_code: u32 = QrcErrors::QuircErrorDataEcc as u32;
                            if let Error::Decode(_) = err {
                            let mut quirc = QrCoder::new().unwrap();
                                println!("this is the case with noise");
                                let image = blur(&image, BLUR_SIZE); 
                                // blur actually not working in unsharpen function
                                // TODO investigate why
                                let image = unsharpen(&image, 0.01, 20);
                                let codes = quirc.codes(&image, width, height).unwrap();
                                extract_code(codes);
                            }
                        },
                    }
                }
            });
            dbg!(Instant::now() - start);
        }
    })
    .unwrap()
}
