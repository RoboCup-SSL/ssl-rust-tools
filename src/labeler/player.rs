use super::reader::LabelerDataReader;
use crate::protos::log_labeler_data;
use crossbeam::channel::{unbounded, TryRecvError};
use protobuf::Message;
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::io::{Read, Seek};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::{io, thread, time};

enum PlayerThreadCommand {
    Stop,
    SetFrameGroup(log_labeler_data::LabelerFrameGroup),
}

pub struct Player<T: Read + Seek> {
    reader: LabelerDataReader<T>,
    player_thread: Option<thread::JoinHandle<()>>,
    control_channel: crossbeam::channel::Sender<PlayerThreadCommand>,
}

fn make_player_thread() -> (
    thread::JoinHandle<()>,
    crossbeam::channel::Sender<PlayerThreadCommand>,
) {
    let (sender, receiver) = unbounded();

    let player_thread = PlayerThread::start(receiver);

    (player_thread, sender)
}

impl<T: Read + Seek> Player<T> {
    pub fn new(reader: LabelerDataReader<T>) -> Player<T> {
        let (player_thread, control_channel) = make_player_thread();

        Player {
            reader,
            player_thread: Some(player_thread),
            control_channel,
        }
    }

    pub fn start(&mut self) {
        if !self.is_running() {
            let (player_thread, control_channel) = make_player_thread();
            self.player_thread = Some(player_thread);
            self.control_channel = control_channel;
        }
    }

    pub fn is_running(&self) -> bool {
        self.player_thread.is_some()
    }

    pub fn stop(&mut self) -> thread::Result<()> {
        match self.player_thread.take() {
            Some(player_thread) => {
                self.control_channel
                    .send(PlayerThreadCommand::Stop)
                    .unwrap();
                player_thread.join()
            }
            None => Ok(()),
        }
    }

    pub fn play_frame(&self, frame_index: usize) {
        if self.is_running() {
            // grab the specified frame index and send it to the
            // player thread
            let frame_group_msg = self.reader.get(frame_index).unwrap();
            self.control_channel
                .send(PlayerThreadCommand::SetFrameGroup(frame_group_msg))
                .unwrap();
        }
    }

    pub fn into_play(self) -> PlayerIntoIterator<T> {
        PlayerIntoIterator::full_range(self)
    }

    pub fn into_play_from(self, start_index: usize) -> PlayerIntoIterator<T> {
        PlayerIntoIterator::range_from(self, start_index)
    }

    pub fn into_play_range(self, start_index: usize, end_index: usize) -> PlayerIntoIterator<T> {
        PlayerIntoIterator::range_from_to(self, start_index, end_index)
    }

    pub fn play(&self) -> PlayerIterator<T> {
        PlayerIterator::full_range(self)
    }

    pub fn play_from(&self, start_index: usize) -> PlayerIterator<T> {
        PlayerIterator::range_from(self, start_index)
    }

    pub fn play_range(&self, start_index: usize, end_index: usize) -> PlayerIterator<T> {
        PlayerIterator::range_from_to(self, start_index, end_index)
    }

    pub fn len(&self) -> usize {
        self.reader.len()
    }
}

impl<T: Read + Seek> Drop for Player<T> {
    fn drop(&mut self) {
        self.stop().unwrap();
    }
}

pub struct PlayerIntoIterator<T: Read + Seek> {
    player: Player<T>,
    index: usize,
    end_index: usize,
}

impl<T: Read + Seek> PlayerIntoIterator<T> {
    fn full_range(player: Player<T>) -> PlayerIntoIterator<T> {
        let end_index = player.len();
        PlayerIntoIterator {
            player,
            index: 0,
            end_index,
        }
    }

    fn range_from_to(
        player: Player<T>,
        start_index: usize,
        end_index: usize,
    ) -> PlayerIntoIterator<T> {
        let end_index = std::cmp::min(end_index, player.len());
        PlayerIntoIterator {
            player,
            index: std::cmp::max(start_index, 0),
            end_index,
        }
    }

    fn range_from(player: Player<T>, start_index: usize) -> PlayerIntoIterator<T> {
        let end_index = player.len();
        PlayerIntoIterator {
            player,
            index: std::cmp::max(start_index, 0),
            end_index,
        }
    }
}

impl<T> Iterator for PlayerIntoIterator<T>
where
    T: Read + Seek,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.end_index {
            return None;
        }

        let index = self.index; // to be returned

        self.player.play_frame(self.index);
        self.index += 1;

        Some(index)
    }
}

pub struct PlayerIterator<'a, T: Read + Seek> {
    player: &'a Player<T>,
    index: usize,
    end_index: usize,
}

impl<'a, T> Iterator for PlayerIterator<'a, T>
where
    T: Read + Seek,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.end_index {
            return None;
        }

        let index = self.index; // to be returned

        self.player.play_frame(self.index);
        self.index += 1;

        Some(index)
    }
}

impl<'a, T: Read + Seek> PlayerIterator<'a, T> {
    fn full_range(player: &Player<T>) -> PlayerIterator<T> {
        let end_index = player.len();
        PlayerIterator {
            player,
            index: 0,
            end_index,
        }
    }

    fn range_from_to(
        player: &Player<T>,
        start_index: usize,
        end_index: usize,
    ) -> PlayerIterator<T> {
        let end_index = std::cmp::min(end_index, player.len());
        PlayerIterator {
            player,
            index: std::cmp::max(start_index, 0),
            end_index,
        }
    }

    fn range_from(player: &Player<T>, start_index: usize) -> PlayerIterator<T> {
        let end_index = player.len();
        PlayerIterator {
            player,
            index: std::cmp::max(start_index, 0),
            end_index,
        }
    }
}

struct PlayerThread {
    control_channel: crossbeam::channel::Receiver<PlayerThreadCommand>,

    // networking sockets
    referee_addr: SocketAddr,
    referee_socket: UdpSocket,
    vision_addr: SocketAddr,
    vision_socket: UdpSocket,
}

//
//  ____  _             _     ____
// / ___|| |_ __ _ _ __| |_  |  _ \ _   _ _ __
// \___ \| __/ _` | '__| __| | | | | | | | '_ \
//  ___) | || (_| | |  | |_  | |_| | |_| | |_) |
// |____/ \__\__,_|_|   \__| |____/ \__,_| .__/
//                                       |_|
//

// TODO(dschwab): this section has code duplicated from
// ssl_log_tools::player.rs. Refactor so all of the multicast
// networking stuff is shared

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

const REFEREE_PORT: u16 = 10003;
const VISION_PORT: u16 = 10006;

lazy_static! {
    static ref REFEREE_ADDR: IpAddr = Ipv4Addr::new(224, 5, 23, 1).into();
    static ref VISION_ADDR: IpAddr = Ipv4Addr::new(224, 5, 23, 2).into();
}

//  _____           _   ____
// | ____|_ __   __| | |  _ \ _   _ _ __
// |  _| | '_ \ / _` | | | | | | | | '_ \
// | |___| | | | (_| | | |_| | |_| | |_) |
// |_____|_| |_|\__,_| |____/ \__,_| .__/
//                                 |_|
//

impl PlayerThread {
    // TODO(dschwab): add ability to override socket addresses
    pub fn start(
        control_channel: crossbeam::channel::Receiver<PlayerThreadCommand>,
    ) -> thread::JoinHandle<()> {
        let referee_addr = SocketAddr::new(*REFEREE_ADDR, REFEREE_PORT);
        let referee_socket = new_sender_socket(&referee_addr).unwrap();
        let vision_addr = SocketAddr::new(*VISION_ADDR, VISION_PORT);
        let vision_socket = new_sender_socket(&vision_addr).unwrap();

        let player_thread = PlayerThread {
            control_channel,
            referee_addr,
            referee_socket,
            vision_addr,
            vision_socket,
        };

        thread::spawn(|| player_thread.player_thread_func())
    }

    // TODO(dschwab): setup some kind of logging, so errors and what
    // not can be detected
    fn player_thread_func(self) {
        let mut curr_frame_msg: Option<log_labeler_data::LabelerFrameGroup> = None;
        loop {
            // check for a new command
            match self.control_channel.try_recv() {
                Ok(command) => match command {
                    PlayerThreadCommand::Stop => {
                        break;
                    }
                    PlayerThreadCommand::SetFrameGroup(frame_group_msg) => {
                        curr_frame_msg = Some(frame_group_msg);
                    }
                },
                Err(error) => match error {
                    TryRecvError::Empty => {
                        // do nothing
                    }
                    TryRecvError::Disconnected => {
                        // this should never happen, but it effectively
                        // means the main player object is gone and this
                        // thread should have been signaled to stop.
                        break;
                    }
                },
            };

            // publish frames in current message
            if let Some(ref frame_msg) = curr_frame_msg {
                for frame in frame_msg.get_frames() {
                    if frame.has_vision_frame() {
                        let msg_bytes = frame
                            .get_vision_frame()
                            .write_to_bytes()
                            .expect("Failed to serialize vision message");
                        self.vision_socket
                            .send_to(&msg_bytes, self.vision_addr)
                            .expect("Could not send vision message");
                    } else if frame.has_referee_frame() {
                        let msg_bytes = frame
                            .get_referee_frame()
                            .write_to_bytes()
                            .expect("Failed to serialize referee message");
                        self.referee_socket
                            .send_to(&msg_bytes, self.referee_addr)
                            .expect("Could not send referee message");
                    } else {
                        // TODO(dschwab): should probably be a warning/error
                    }
                }
            }

            thread::sleep(time::Duration::from_millis(1));
        }
    }
}
