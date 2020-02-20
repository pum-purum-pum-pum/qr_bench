use clap::{App, Arg, SubCommand};
use image::Luma;
use qrcode::{EcLevel, QrCode};
use scan::qr_search;
// another library to try for this task
mod error;
mod scan;

#[cfg(test)]
mod test;

#[macro_use]
extern crate log;
use env_logger::Builder;
use log::LevelFilter;

fn main() {
    let matches = App::new("QRSearcher")
        .version("0.1")
        .about("Seach for QR codes on photos")
        .subcommand(
            SubCommand::with_name("read")
                .about("run on folder with images")
                .arg(
                    Arg::with_name("INPUT_DIR")
                        .help("Set the input directory to use")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name("gen").about("Generate qr").arg(
                Arg::with_name("name")
                    .help("Name of the generated image")
                    .required(true)
                    .index(1),
            ), // .arg(
               //     Arg::with_name("data")
               //         .help("text to encode(without spaces for now)")
               //         .required(true)
               //         .index(2)
               // )
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets verbosity"),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("gen") {
        let name = matches
            .value_of("name")
            .expect("please, provide a name of the gerated image, example: qr_image");
        // let text = matches.value_of("data").expect("please provide text to encode");
        let text = "";
        let data = text.as_bytes();
        let code = QrCode::with_error_correction_level(data, EcLevel::L).unwrap();
        let image = code.render::<Luma<u8>>().build();
        image.save(format!("{}.png", name)).unwrap();
    };
    if let Some(matches) = matches.subcommand_matches("read") {
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
    }
}
