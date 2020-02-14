use std;
use std::fs::{self, File};
use std::path::PathBuf;
use std::io::Read;
use std::str::from_utf8;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use log::{debug};

use crate::error::{QRErrors, Result};

use crossbeam_utils::thread;
use crossbeam::thread::Scope;
use image;
use image::{
    imageops::{blur, unsharpen, FilterType}, GrayImage
};
use quirc::{Codes, QrCoder};
// another library to try for this task
use quirc;
use once_cell::sync::Lazy;

// max number of images to process first(from begining and end of images sequence)
const FIRST_TAKE: usize = 10;
// When handling noisy images use blur with that kernel size for preprocessing
const BLUR_SIZE: f32 = 4.;
const UNSHARPEN_THRESHOLD: i32 = 20;

const RESIZE_WIDTH: u32 = 800;
const RESIZE_HEIGHT: u32 = 600;

// global msg of the qr code synced with threads via Mutex.
static QR_MSG: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
// flag to check from threads if we found qr code
static QR_FOUND: AtomicBool = AtomicBool::new(false);


// perform ordering in which we will process our images:
// [0..n][l - n..l][n, l - n]. n is param, l is lenght of array
pub fn order<T>(names: &mut Vec<T>, first_take: usize) -> Vec<T> {
    let mut ordered_filenames: Vec<_> = names.drain(..first_take).collect();
    let last = names.drain((names.len() - first_take)..);
    ordered_filenames.extend(last);
    ordered_filenames.extend(names.drain(..));
    ordered_filenames
}

#[test]
fn order_test() {
    let mut names = vec!["a", "b", "c", "d"];
    assert_eq!(order(&mut names, 1), ["a", "d", "b", "c"]);
}

// go through codes we found and assign to the global variable if found one
pub fn extract_code(codes: Codes) -> Result<()> {
    for code in codes {
        if let Ok(code) = code {
            QR_FOUND.compare_and_swap(false, true, Ordering::SeqCst);
            *QR_MSG.lock()? = Some(from_utf8(&code.payload)?.to_string());
            break; // we only need (first) one
        }
    }
    Ok(())
}

// try to detect qr. If found write in global variable
pub fn detection(image: GrayImage) -> Result<()> {
    debug!("start detection");
    let width = image.width();
    let height = image.height();
    if QR_FOUND.load(Ordering::SeqCst) {
        // QR already found
        return Ok(());
    }
    let mut quirc = QrCoder::new().map_err(|_| QRErrors::QrDetectError)?;
    let codes = quirc.codes(&image, width, height).map_err(|_| QRErrors::QrDetectError)?;
    for code in codes {
        match code {
            Ok(code) => {
                QR_FOUND.compare_and_swap(false, true, Ordering::SeqCst);
                *QR_MSG.lock()? = Some(from_utf8(&code.payload)?.to_string());
            }
            Err(err) => {
                if let quirc::Error::Decode(_) = err {
                    let mut quirc = QrCoder::new().map_err(|_| QRErrors::QrDetectError)?;
                    let image = blur(&image, BLUR_SIZE);
                    // blur actually not working in unsharpen function
                    // TODO investigate why
                    let image = unsharpen(&image, 0.01, UNSHARPEN_THRESHOLD);
                    let codes = quirc.codes(&image, width, height).map_err(|_| QRErrors::QrDetectError)?;
                    let _err = extract_code(codes); // if ok it will extract value in global msg
                }
            }
        }
    }
    Ok(())
}

pub fn laod_and_detect(path: &PathBuf, scope: &Scope) -> Result<()> {
    if QR_FOUND.load(Ordering::SeqCst) {
        // QR already found
        return Ok(());
    }
    let mut file = File::open(path.clone())?;
    let mut vec = Vec::new();
    file.read_to_end(&mut vec)?;
    let image = image::load_from_memory(&vec)?
        .resize(RESIZE_WIDTH, RESIZE_HEIGHT, FilterType::Nearest)
        .to_luma();
    let _handle = scope.spawn(move |_| {
        debug!("timestamp {:?}", Instant::now());
        let _err = detection(image); // ignoring detection error, searching further
    });
    Ok(())
}

// search for qr in the directory with images
pub fn qr_search(dir_name: &str) -> Result<String> {
    // collect files
    let mut file_names = vec![];
    for entry in fs::read_dir(dir_name)? {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                continue;
            };
            file_names.push(path);
        }
    }
    // sort, take the first N, the last N and the rest middle
    file_names.sort();
    let first_take = ((0.1 * (file_names.len() as f32)) as usize).max(1).min(FIRST_TAKE);
    let ordered_filenames = order(&mut file_names, first_take);

    // run scan in the directory, !ignoring panics in threads
    let _err = thread::scope(|s| {
        for path in ordered_filenames.iter() {
            let _err = laod_and_detect(&path, s);
        }
    });
    if let Some(msg) = &*QR_MSG.lock()? {
        Ok(msg.clone())
    } else {
        Err(QRErrors::QrSerachError)?
    }
}