use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use ssl_log_tools::labeler::writer;
use ssl_log_tools::persistence::reader;
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

fn main() {
    let matches = App::new("Create a log labeler data file.")
        .version("1.0")
        .author("Devin Schwab <dschwab@andrew.cmu.edu>")
        .about("Pre-process a log file before labeling.")
        .arg(
            Arg::with_name("LOG_FILE")
                .help("Path to the log file to pre-process. Use '-' for stdin.")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("LABELER_DATA_FILE")
                .help("Path to save the pre-processed labeler data file to.")
                .required(true)
                .index(2),
        )
        .get_matches();

    let (input, input_len): (Box<BufRead>, Option<u64>) =
        match matches.value_of("LOG_FILE").unwrap() {
            "-" => (Box::new(BufReader::new(io::stdin())), None),
            log_path => {
                let file = fs::File::open(log_path).expect("Failed to open input file");
                let file_len = file
                    .metadata()
                    .expect("Failed to get input file metdata")
                    .len();
                (Box::new(BufReader::new(file)), Some(file_len))
            }
        };

    let prog_bar = match input_len {
        Some(len) => {
            let prog_bar = ProgressBar::new(len);
            prog_bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} {msg} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({elapsed_precise}/{eta_precise})")
                      );
            prog_bar
        }
        None => {
            let prog_bar = ProgressBar::new_spinner();
            prog_bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} {msg} {bytes} {elapsed_precise}"),
            );
            prog_bar
        }
    };
    let reader = prog_bar.wrap_read(input);

    let output_path = Path::new(matches.value_of("LABELER_DATA_FILE").unwrap());

    let output_path_parent = output_path
        .parent()
        .expect("Unable to get parent directory of output path");
    fs::create_dir_all(output_path_parent).expect("Failed to create output directory");

    let log_reader = reader::LogReader::new(reader).expect("Could not read log file");
    let mut labeler_data_writer = writer::LabelerDataWriter::new_from_path(output_path)
        .expect("Could not write labeler data file");

    prog_bar.set_message("Processing log file");
    for message in log_reader.filter_map(Result::ok) {
        labeler_data_writer
            .add_msg(message)
            .expect("Failed to write message to labeler data writer");
    }
    prog_bar.finish_with_message("Finished pre-processing.");
}
