use clap::{App, Arg};
use ssl_rust_tools::labeler::reader::LabelerDataReader;
use std::path::Path;

fn main() {
    let matches = App::new("Add num passing and goal shot events to the labeler data file.")
        .version("1.0")
        .author("Devin Schwab <dschwab@andrew.cmu.edu>")
        .about("Add info about num passing and goal shots to saved labeler file")
        .arg(
            Arg::with_name("LABELER_FILE")
                .help("Path to labeler file")
                .required(true)
                .index(1),
        )
        .get_matches();

    // if this section has an error then the file format is not correct
    let log_path = Path::new(matches.value_of("LABELER_FILE").unwrap());
    let reader = LabelerDataReader::new_from_path(log_path).unwrap();

    println!("Num Cameras: {}", reader.num_cameras());
    println!("Num Messages: {}", reader.len());
    println!("Num Passing Events: {}", reader.num_passing_events());
    println!("Num Goal Shot Events: {}", reader.num_goal_shot_events());
}
