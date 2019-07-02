use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use clap::{App, Arg};
use protobuf::Message;
use ssl_rust_tools::labeler::reader::LabelerDataReader;
use ssl_rust_tools::protos::log_labeler_data;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
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
        .arg(
            Arg::with_name("NUM_PASSING_EVENTS")
                .help("Number of passing events")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("NUM_GOAL_SHOT_EVENTS")
                .help("Number of goal shot events")
                .required(true)
                .index(3),
        )
        .get_matches();

    // if this section has an error then the file format is not correct
    {
        let log_path = Path::new(matches.value_of("LABELER_FILE").unwrap());
        LabelerDataReader::new_from_path(log_path).unwrap();
    }

    let log_path = Path::new(matches.value_of("LABELER_FILE").unwrap());
    let mut log_file = OpenOptions::new()
        .write(true)
        .create(false)
        .truncate(false)
        .read(true)
        .open(log_path)
        .unwrap();

    // read metadata message
    let metadata_size_offset = -(std::mem::size_of::<u32>() as i64);
    log_file.seek(SeekFrom::End(metadata_size_offset)).unwrap();
    let metadata_msg_size = log_file.read_u32::<BigEndian>().unwrap() as usize;
    log_file
        .seek(SeekFrom::End(
            metadata_size_offset - (metadata_msg_size as i64),
        ))
        .unwrap();

    // parse to protobuf
    let mut metadata_msg_bytes = vec![0u8; metadata_msg_size];
    log_file.read_exact(&mut metadata_msg_bytes).unwrap();
    let mut metadata =
        protobuf::parse_from_bytes::<log_labeler_data::LabelerMetadata>(&metadata_msg_bytes)
            .unwrap();

    // set num events fields
    let num_passing_events = matches
        .value_of("NUM_PASSING_EVENTS")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let num_goal_shot_events = matches
        .value_of("NUM_GOAL_SHOT_EVENTS")
        .unwrap()
        .parse::<u32>()
        .unwrap();

    metadata.set_num_passing_events(num_passing_events);
    metadata.set_num_goal_shot_events(num_goal_shot_events);

    // write new metadata message
    log_file
        .seek(SeekFrom::End(
            metadata_size_offset - (metadata_msg_size as i64),
        ))
        .unwrap();
    let metadata_bytes = metadata.write_to_bytes().unwrap();
    log_file.write_all(&metadata_bytes).unwrap();
    log_file
        .write_u32::<BigEndian>(metadata_bytes.len() as u32)
        .unwrap();
}
