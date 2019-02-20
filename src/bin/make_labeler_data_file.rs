use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use ssl_log_tools::labeler::writer;
use ssl_log_tools::persistence::reader;
use std::fs;
use std::path::Path;

fn main() {
    let matches = App::new("Create a log labeler data file.")
        .version("1.0")
        .author("Devin Schwab <dschwab@andrew.cmu.edu>")
        .about("Pre-process a log file before labeling.")
        .arg(
            Arg::with_name("LOG_FILE")
                .help("Path to the log file to pre-process.")
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

    let log_path = Path::new(matches.value_of("LOG_FILE").unwrap());
    let output_path = Path::new(matches.value_of("LABELER_DATA_FILE").unwrap());

    let output_path_parent = output_path
        .parent()
        .expect("Unable to get parent directory of output path");
    fs::create_dir_all(output_path_parent).expect("Failed to create output directory");

    let file = fs::File::open(log_path).expect("Failed to open log file");
    let prog_bar = ProgressBar::new(file.metadata().expect("Failed to get file metadata").len());
    prog_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} {msg} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({elapsed_precise}/{eta_precise})")
    );
    let file = prog_bar.wrap_read(file);

    let log_reader = reader::LogReader::new(file).expect("Could not read log file");
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
