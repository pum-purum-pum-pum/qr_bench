use scan::qr_search;

use clap::{App, Arg};
// another library to try for this task
mod error;
mod scan;

#[macro_use]
extern crate log;
use env_logger::Builder;
use log::LevelFilter;

fn main() {
    let matches = App::new("QRSearcher")
        .version("0.1")
        .about("Seach for QR codes on photos")
        .arg(
            Arg::with_name("INPUT_DIR")
                .help("Set the input directory to use")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets verbosity"),
        )
        .get_matches();
    let dir_name = matches
        .value_of("INPUT_DIR")
        .expect("Directory for search not specified.");
    let mut builder = Builder::new();
    let verbosity = matches.occurrences_of("v");
    if verbosity > 0 {
        builder.filter_level(LevelFilter::Debug).init();
    } else {
        builder.init();
    }

    debug!("debug message");

    if let Ok(msg) = qr_search(dir_name) {
        println!("{}", msg);
    } else {
        println!("not found");
    }
    //     TODO generator if needed
    //
    // if false {
    //     use image;
    //     use image::Luma;
    //     use qrcode::{types::EcLevel, QrCode};
    //     let data = &b"hello world"[..];
    //     let code = QrCode::with_error_correction_level(data, EcLevel::L).unwrap();
    //     let image = code.render::<Luma<u8>>().build();
    //     image.save("qr_generated.png").unwrap();
    // }
}
