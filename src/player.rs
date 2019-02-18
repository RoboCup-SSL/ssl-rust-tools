use crate::persistence::message::MessageType;
use crate::persistence::reader;
use crate::protos::messages_robocup_ssl_referee::SSL_Referee_Stage;
use chrono::prelude::*;
use protobuf::Message as ProtobufMessage;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::io;
use std::io::{Read, Seek};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::thread;

pub const REFEREE_PORT: u16 = 10003;
pub const VISION_PORT: u16 = 10006;

lazy_static! {
    pub static ref REFEREE_ADDR: IpAddr = Ipv4Addr::new(224, 5, 23, 1).into();
    pub static ref VISION_ADDR: IpAddr = Ipv4Addr::new(224, 5, 23, 2).into();
}

pub struct PlayerOptions {
    referee_addr: IpAddr,
    referee_port: u16,
    vision_addr: IpAddr,
    vision_port: u16,
}

impl Default for PlayerOptions {
    fn default() -> Self {
        PlayerOptions {
            referee_addr: *REFEREE_ADDR,
            referee_port: REFEREE_PORT,
            vision_addr: *VISION_ADDR,
            vision_port: VISION_PORT,
        }
    }
}

#[derive(Debug, Fail)]
pub enum PlayerError {
    #[fail(display = "Only Ipv4 is supported")]
    OnlyIpv4Supported,
    #[fail(display = "{}", _0)]
    Io(#[fail(cause)] io::Error),
}

impl From<io::Error> for PlayerError {
    fn from(error: io::Error) -> Self {
        PlayerError::Io(error)
    }
}

pub struct Player<T: Read + Seek> {
    reader: reader::LogReader<T>,
    referee_addr: SocketAddr,
    referee_socket: UdpSocket,
    vision_addr: SocketAddr,
    vision_socket: UdpSocket,
}

fn new_socket(addr: &SocketAddr) -> io::Result<Socket> {
    // currently all the SSL software is IPv4 only
    assert!(addr.is_ipv4());

    let domain = Domain::ipv4();
    let socket = Socket::new(domain, Type::dgram(), Some(Protocol::udp()))?;

    Ok(socket)
}

fn new_sender_socket(addr: &SocketAddr) -> io::Result<UdpSocket> {
    // currently only support IPv4
    assert!(addr.is_ipv4());

    let socket = new_socket(addr)?;

    socket.set_multicast_if_v4(&Ipv4Addr::new(0, 0, 0, 0))?;
    socket.bind(&SockAddr::from(SocketAddr::new(
        Ipv4Addr::new(0, 0, 0, 0).into(),
        0, // use a random sender port
    )))?;

    Ok(socket.into_udp_socket())
}

impl<T: Read + Seek> Player<T> {
    pub fn new_with_options(
        reader: reader::LogReader<T>,
        options: PlayerOptions,
    ) -> Result<Player<T>, PlayerError> {
        let referee_addr = SocketAddr::new(options.referee_addr, options.referee_port);
        let referee_socket = new_sender_socket(&referee_addr)?;
        let vision_addr = SocketAddr::new(options.vision_addr, options.vision_port);
        let vision_socket = new_sender_socket(&vision_addr)?;

        Ok(Player {
            reader,
            referee_addr,
            referee_socket,
            vision_addr,
            vision_socket,
        })
    }

    pub fn new(reader: reader::LogReader<T>) -> Result<Player<T>, PlayerError> {
        Player::new_with_options(reader, PlayerOptions::default())
    }

    pub fn play(self) {
        self.play_at_speed(1.0f32)
    }

    pub fn play_at_speed(self, speed: f32) {
        // TOOD(dschwab): Should try and implement a non-consuming
        // iterator. Will probably need to reopen the same log file,
        // seek to the same initial position and then iterate on that,
        // so that the file cursor isn't broken when multiple
        // iterators exist.
        let mut curr_stage: Option<SSL_Referee_Stage> = None;

        let mut start_time = Utc::now();
        let mut ref_timestamp = 0i64;
        for message in self.reader.filter_map(Result::ok) {
            if is_running_stage(curr_stage) {
                if ref_timestamp != 0 {
                    let real_elapsed = (Utc::now() - start_time)
                        .num_nanoseconds()
                        .expect("real_elapsed overflowed i64 nanoseconds");
                    let msg_elapsed = ((message.timestamp - ref_timestamp) as f32 / speed) as i64;                    
                    let sleep_time = msg_elapsed - real_elapsed;
                    println!("sleep_time: {}", sleep_time);
                    if sleep_time > 0 {
                        let sleep_time = chrono::Duration::nanoseconds(sleep_time);
                        thread::sleep(
                            sleep_time
                                .to_std()
                                .expect("Failed to convert to std::time::Duration"),
                        );
                    }
                } else {
                    start_time = Utc::now();
                    ref_timestamp = message.timestamp;
                }

                match message.msg_type {
                    MessageType::Blank | MessageType::Vision2010(_) | MessageType::Unknown(_) => {
                        // TODO(dschwab): print log message
                        println!("Skipping message");
                    }
                    MessageType::Refbox2013(ref ref_msg) => {
                        let msg_bytes = ref_msg
                            .write_to_bytes()
                            .expect("Failed to serialize ref message");
                        self.referee_socket
                            .send_to(&msg_bytes, self.referee_addr)
                            .expect("Could not send ref message");
                    }
                    MessageType::Vision2014(ref vision_msg) => {
                        let msg_bytes = vision_msg
                            .write_to_bytes()
                            .expect("Failed to serialize vision message");
                        self.vision_socket
                            .send_to(&msg_bytes, self.vision_addr)
                            .expect("Could not send vision message");
                    }
                };
            } else {
                ref_timestamp = 0;
            }

            if let MessageType::Refbox2013(ref ref_msg) = message.msg_type {
                curr_stage = Some(ref_msg.get_stage());
            }
        }
    }
}

fn is_running_stage(stage: Option<SSL_Referee_Stage>) -> bool {
    match stage {
        Some(stage) => match stage {
            SSL_Referee_Stage::NORMAL_FIRST_HALF
            | SSL_Referee_Stage::NORMAL_SECOND_HALF
            | SSL_Referee_Stage::EXTRA_FIRST_HALF
            | SSL_Referee_Stage::EXTRA_SECOND_HALF => true,
            _ => false,
        },
        None => true,
    }
}
