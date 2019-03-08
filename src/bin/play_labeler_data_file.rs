use clap::{App, Arg};
use indicatif::{ProgressBar, ProgressStyle};
use ssl_rust_tools::labeler::player;
use ssl_rust_tools::labeler::reader::LabelerDataReader;
use std::path::Path;
use std::{thread, time};

fn main() {
    let matches = App::new("Play labeler data")
        .version("1.0")
        .author("Devin Schwab <dschwab@andrew.cmu.edu>")
        .about("Play an Labeler Data File.")
        .arg(
            Arg::with_name("LABELER_DATA_FILE")
                .help("Specifies labeler data file that should be played")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("from")
                .short("f")
                .long("from")
                .value_name("FROM")
                .help("Play from specified frame.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("to")
                .short("t")
                .long("to")
                .value_name("TO")
                .help("Play until specified frame.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("speed")
                .short("s")
                .long("speed")
                .value_name("SPEED")
                .help("Play with speedup factor, defaults to 1.0 which is real-time.")
                .takes_value(true),
        )
        .get_matches();

    let speed = matches.value_of("speed").unwrap_or("1.0");
    let speed = speed.parse::<f32>().unwrap();

    let log_path = Path::new(matches.value_of("LABELER_DATA_FILE").unwrap());
    let reader = LabelerDataReader::new_from_path(log_path).unwrap();
    let player = player::Player::new(reader);

    let prog_bar = ProgressBar::new(player.len() as u64);
    prog_bar.set_style(
        ProgressStyle::default_bar().template("[{wide_bar:.cyan/blue}] ({elapsed_precise})"),
    );

    let from_index = matches.value_of("from").unwrap_or("0");
    let from_index = from_index.parse::<usize>().unwrap();

    let to_index = match matches.value_of("to") {
        Some(value) => value.parse::<usize>().unwrap(),
        None => player.len(),
    };

    let sleep_time = time::Duration::from_millis((16. / speed) as u64);
    
    prog_bar.set_position(from_index as u64);
    for _ in prog_bar.wrap_iter(player.into_play_range(from_index, to_index)) {
        thread::sleep(sleep_time);
    }
}
