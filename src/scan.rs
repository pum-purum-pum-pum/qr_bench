use std;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;
use std::str::from_utf8;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use log::debug;

use crate::error::{QRErrors, Result};

use crossbeam::thread::Scope;
use crossbeam_utils::thread;
use image;
use image::{
    imageops::{blur, unsharpen, FilterType},
    GrayImage, Luma,
};
use quirc::{Codes, QrCoder};
// another library to try for this task
use once_cell::sync::Lazy;
use quirc;

// max number of images to process first(from begining and end of images sequence)
const FIRST_TAKE: usize = 10;
// When handling noisy images use blur with that kernel size for preprocessing
const BLUR_SIZE: f32 = 4.;
const UNSHARPEN_THRESHOLD: i32 = 200;

const RESIZE_WIDTH: u32 = 800;
const RESIZE_HEIGHT: u32 = 600;

// global msg of the qr code synced with threads via Mutex.
pub static QR_MSG: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));
// flag to check from threads if we found qr code
pub static QR_FOUND: AtomicBool = AtomicBool::new(false);

// perform ordering in which we will process our images:
// [0..n][l - n..l][n, l - n]. n is param, l is lenght of array
pub fn order<T: Clone>(names: &mut Vec<T>, first_take: usize) -> Vec<T> {
    if 2 * first_take > names.len() {
        return names.clone();
    }
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

#[test]
fn empty_order() {
    let mut names: Vec<&str> = vec![];
    order(&mut names, 1);
}

// go through codes we found and assign to the global variable if found one
pub fn extract_code(codes: Codes) -> Result<String> {
    let mut result = None;
    for code in codes {
        if let Ok(code) = code {
            let msg = from_utf8(&code.payload)?.to_string();
            result = Some(msg.clone());
            QR_FOUND.compare_and_swap(false, true, Ordering::SeqCst);
            *QR_MSG.lock()? = Some(msg);
            break; // we only need (first) one
        }
    }
    result.ok_or(Box::new(QRErrors::QrDetectError))
}

// try to detect qr. If found write in global variable
pub fn detection(image: GrayImage) -> Result<String> {
    debug!("start detection");
    let width = image.width();
    let height = image.height();

    // let image = blur(&image, BLUR_SIZE);
    // // blur actually not working in unsharpen function(we are doing it separetly)
    // // TODO investigate why
    // let image = unsharpen(&image, 0.01, UNSHARPEN_THRESHOLD);

    if QR_FOUND.load(Ordering::SeqCst) {
        // QR already found
        return Err(Box::new(QRErrors::QrAlreadyFound));
    }
    let mut quirc = QrCoder::new().map_err(|_| QRErrors::QrDetectError)?;
    let codes = quirc
        .codes(&image, width, height)
        .map_err(|_| QRErrors::QrDetectError)?;
    let mut result = None;
    for code in codes {
        match code {
            Ok(code) => {
                QR_FOUND.compare_and_swap(false, true, Ordering::SeqCst);
                let code = from_utf8(&code.payload)?.to_string();
                result = Some(code.clone());
                *QR_MSG.lock()? = Some(code);
            }
            Err(err) => {
                // the case when we detect qr but failed to parse it
                // possible try more then one filter and detect here
                if let quirc::Error::Decode(_) = err {}
            }
        }
    }
    if result.is_none() {
        let mut quirc = QrCoder::new().map_err(|_| QRErrors::QrDetectError)?;
        let image = blur(&image, BLUR_SIZE);
        // blur actually not working in unsharpen function(we are doing it separetly)
        // TODO investigate why
        let image = unsharpen(&image, 0.01, UNSHARPEN_THRESHOLD);
        let codes = quirc
            .codes(&image, width, height)
            .map_err(|_| QRErrors::QrDetectError)?;
        let code = extract_code(codes); // if ok it will extract value in global msg
        if let Ok(code) = code {
            result = Some(code)
        }
    }
    result.ok_or(Box::new(QRErrors::QrDetectError))
}

pub fn load_resized_luma(path: &PathBuf) -> Result<image::ImageBuffer<Luma<u8>, Vec<u8>>> {
    let mut file = File::open(path.clone())?;
    let mut vec = Vec::new();
    file.read_to_end(&mut vec)?;
    let image = image::load_from_memory(&vec)?
        .resize(RESIZE_WIDTH, RESIZE_HEIGHT, FilterType::Nearest)
        .to_luma();
    Ok(image)
}

pub fn load_and_detect(path: &PathBuf, scope: &Scope) -> Result<()> {
    if QR_FOUND.load(Ordering::SeqCst) {
        // QR already found
        return Ok(());
    }
    let image = load_resized_luma(&path)?;
    let _handle = scope.spawn(move |_| {
        debug!("timestamp {:?}", Instant::now());
        let _err = detection(image); // ignoring detection error, searching further
        debug!("detection {:?}", _err);
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
    let first_take = ((0.1 * (file_names.len() as f32)) as usize)
        .max(1)
        .min(FIRST_TAKE);
    let ordered_filenames = order(&mut file_names, first_take);

    // run scan in the directory, !ignoring panics in threads
    let _err = thread::scope(|s| {
        for path in ordered_filenames.iter() {
            let _err = load_and_detect(&path, s);
        }
    });
    if let Some(msg) = &*QR_MSG.lock()? {
        Ok(msg.clone())
    } else {
        Err(QRErrors::QrSerachError.into())
    }
}
