use clap::{App, Arg};
use ssl_log_tools::persistence::reader;
use ssl_log_tools::player;
use std::path::Path;

fn main() {
    let matches = App::new("Play Log")
        .version("1.0")
        .author("Devin Schwab <dschwab@andrew.cmu.edu>")
        .about("Play an SSL RoboCup log.")
        .arg(
            Arg::with_name("speed")
                .short("s")
                .long("speed")
                .value_name("SPEED")
                .help("Sets playback speed, defaults to 1.0 which is real-time.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("LOG_FILE")
                .help("Specifies log file that should be played")
                .required(true)
                .index(1),
        )
        .get_matches();

    let speed = matches.value_of("speed").unwrap_or("1.0");
    let speed = speed.parse::<f32>().unwrap();
    let log_path = Path::new(matches.value_of("LOG_FILE").unwrap());
    let reader = reader::LogReader::new_from_path(log_path).unwrap();    
    let player = player::Player::new(reader).unwrap();


    println!("Playing with speed: {}", speed);
    player.play_at_speed(speed);
}
