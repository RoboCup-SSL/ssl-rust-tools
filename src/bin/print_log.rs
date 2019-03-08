extern crate clap;
extern crate ssl_rust_tools;
use clap::{App, Arg};
use ssl_rust_tools::persistence::reader;
use std::path::Path;

fn main() {
    let matches = App::new("Print Log")
        .version("1.0")
        .author("Devin Schwab <dschwab@andrew.cmu.edu>")
        .about("Prints an SSL RoboCup log.")
        .arg(
            Arg::with_name("LOG_FILE")
                .help("Specifies log file that should be printed")
                .required(true)
                .index(1),
        )
        .get_matches();

    let log_path = Path::new(matches.value_of("LOG_FILE").unwrap());
    let reader = reader::LogReader::new_from_path(log_path).unwrap();
    for message in reader {
        println!("{:#?}", message);
    }
}
