use crate::protos::messages_robocup_ssl_referee;
use crate::protos::messages_robocup_ssl_wrapper;
use protobuf;
use std::io;
use std::io::Read;

#[derive(Debug, Fail)]
pub enum MessageError {
    // If msg size < 0
    #[fail(display = "Invalid message size {}", msg_size)]
    InvalidMessageSize { msg_size: i32 },
    #[fail(display = "{}", _0)]
    Protobuf(#[fail(cause)] protobuf::error::ProtobufError),
    #[fail(display = "{}", _0)]
    Io(#[fail(cause)] io::Error),
    #[fail(display = "Blank message size not 0. Got {}", _0)]
    NonZeroSizeBlankMsg { msg_size: i32 },
}

impl From<io::Error> for MessageError {
    fn from(error: io::Error) -> Self {
        MessageError::Io(error)
    }
}

impl From<protobuf::ProtobufError> for MessageError {
    fn from(error: protobuf::ProtobufError) -> Self {
        MessageError::Protobuf(error)
    }
}

const BLANK_TYPE: i32 = 0;
const UNKNOWN_TYPE: i32 = 1;
const VISION2010_TYPE: i32 = 2;
const REFBOX2013_TYPE: i32 = 3;
const VISION2014_TYPE: i32 = 4;

#[derive(Debug)]
pub enum MessageType {
    // Empty message
    Blank,
    // Unsupported Message Type. Just return the raw protobuf data so
    // the user can do more with it if wanted
    Vision2010(Vec<u8>),
    // Parse the message for the user and return the parsed protobuf
    // message
    Refbox2013(messages_robocup_ssl_referee::SSL_Referee),
    // Main message type. Parse for the user and return the type
    Vision2014(messages_robocup_ssl_wrapper::SSL_WrapperPacket),
    // Could be a message from the future, that is unsupported by this
    // version of the tool. Just return the raw bytes and let the user
    // deal with it
    Unknown(Vec<u8>),
}

#[derive(Debug)]
pub struct Message {
    // receiver timestamp in ns
    pub timestamp: i64,
    // enum which contains the parse message or bytes depending on the
    // type
    pub msg_type: MessageType,
}

impl Message {
    // TODO(dschwab): If I add Seek trait bound, can automatically
    // rewind if message parsing fails
    pub fn parse_from_reader<T: Read>(reader: &mut T) -> Result<Message, MessageError> {
        let mut timestamp = [0u8; 8];
        reader.read_exact(&mut timestamp)?;
        let timestamp: i64 = unsafe { std::mem::transmute(timestamp) };

        let mut msg_type = [0u8; 4];
        reader.read_exact(&mut msg_type)?;
        let msg_type: i32 = unsafe { std::mem::transmute(msg_type) };

        let mut msg_size = [0u8; 4];
        reader.read_exact(&mut msg_size)?;
        let msg_size: i32 = unsafe { std::mem::transmute(msg_size) };
        if msg_size < 0 {
            return Err(MessageError::InvalidMessageSize { msg_size });
        }

        // TODO(dschwab): Parse message type
        match msg_type {
            BLANK_TYPE => Ok(Message::parse_blank_msg_from_reader(
                reader, timestamp, msg_size,
            )?),
            VISION2010_TYPE => Ok(Message::parse_vision_2010_msg_from_reader(
                reader, timestamp, msg_size,
            )?),
            REFBOX2013_TYPE => Ok(Message::parse_refbox_2013_msg_from_reader(
                reader, timestamp, msg_size,
            )?),
            VISION2014_TYPE => Ok(Message::parse_vision_2014_msg_from_reader(
                reader, timestamp, msg_size,
            )?),
            // Makes this future proof by just returning the specified
            // bytes unparsed, even if msg_type number is outside of
            // expected range.
            UNKNOWN_TYPE | _ => Ok(Message::parse_unknown_msg_from_reader(
                reader, timestamp, msg_size,
            )?),
        }
    }

    fn parse_blank_msg_from_reader<T: Read>(
        _reader: &mut T,
        timestamp: i64,
        msg_size: i32,
    ) -> Result<Message, MessageError> {
        if msg_size == 0 {
            Ok(Message {
                timestamp,
                msg_type: MessageType::Blank,
            })
        } else {
            Err(MessageError::NonZeroSizeBlankMsg { msg_size })
        }
    }

    fn parse_unknown_msg_from_reader<T: Read>(
        reader: &mut T,
        timestamp: i64,
        msg_size: i32,
    ) -> Result<Message, MessageError> {
        let mut msg_bytes = vec![0u8; msg_size as usize];
        reader.read_exact(&mut msg_bytes)?;

        Ok(Message {
            timestamp,
            msg_type: MessageType::Unknown(msg_bytes),
        })
    }

    fn parse_vision_2010_msg_from_reader<T: Read>(
        reader: &mut T,
        timestamp: i64,
        msg_size: i32,
    ) -> Result<Message, MessageError> {
        let mut msg_bytes = vec![0u8; msg_size as usize];
        reader.read_exact(&mut msg_bytes)?;

        Ok(Message {
            timestamp,
            msg_type: MessageType::Vision2010(msg_bytes),
        })
    }

    fn parse_refbox_2013_msg_from_reader<T: Read>(
        reader: &mut T,
        timestamp: i64,
        msg_size: i32,
    ) -> Result<Message, MessageError> {
        let mut msg_bytes = vec![0u8; msg_size as usize];
        reader.read_exact(&mut msg_bytes)?;

        let refbox_msg =
            protobuf::parse_from_bytes::<messages_robocup_ssl_referee::SSL_Referee>(&msg_bytes)?;
        Ok(Message {
            timestamp,
            msg_type: MessageType::Refbox2013(refbox_msg),
        })
    }

    fn parse_vision_2014_msg_from_reader<T: Read>(
        reader: &mut T,
        timestamp: i64,
        msg_size: i32,
    ) -> Result<Message, MessageError> {
        let mut msg_bytes = vec![0u8; msg_size as usize];
        reader.read_exact(&mut msg_bytes)?;

        let vision_msg = protobuf::parse_from_bytes::<
            messages_robocup_ssl_wrapper::SSL_WrapperPacket,
        >(&msg_bytes)?;

        Ok(Message {
            timestamp,
            msg_type: MessageType::Vision2014(vision_msg),
        })
    }

    pub fn write_to_vec(&self, v: &mut Vec<u8>) -> Result<(), MessageError> {
        Ok(())
    }

    pub fn write_to_bytes(&self) -> Result<Vec<u8>, MessageError> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    extern crate proptest;
    extern crate tempfile;

    use super::*;
    use proptest::prelude::*;
    use protobuf::Message as ProtobufMessage;
    use std::io;
    use std::io::{Seek, Write};
    use crate::test_utils::protos::*;
    use crate::test_utils::message::*;

    impl From<MessageError> for TestCaseError {
        fn from(error: MessageError) -> Self {
            TestCaseError::fail(format!("{}", error))
        }
    }

    fn write_msg<Writer: Write>(
        writer: &mut Writer,
        timestamp: i64,
        msg_type: i32,
        msg_size: i32,
        msg_bytes: &[u8],
    ) -> Result<(), TestCaseError> {
        let timestamp_bytes: [u8; 8] = unsafe { std::mem::transmute(timestamp) };
        writer.write(&timestamp_bytes)?;
        let msg_type_bytes: [u8; 4] = unsafe { std::mem::transmute(msg_type) };
        writer.write(&msg_type_bytes)?;
        let msg_size_bytes: [u8; 4] = unsafe { std::mem::transmute(msg_size) };
        writer.write(&msg_size_bytes)?;
        writer.write(msg_bytes)?;

        Ok(())
    }


    proptest! {
        #[test]
        fn parse_bad_size(timestamp in 0..std::i64::MAX, msg_type in 0..5i32, bad_size in std::i32::MIN..0) {
            // create temp file with the bad msg_size number
            let mut tmpfile = tempfile::tempfile()?;
            write_msg(&mut tmpfile, timestamp, msg_type, bad_size, &[])?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match Message::parse_from_reader(&mut tmpfile) {
                Ok(_) => {
                    let message = format!("Message parsed correctly when it shouldn't.");
                    return Err(TestCaseError::fail(message));
                },
                Err(err) => match err {
                    MessageError::InvalidMessageSize { msg_size } => {
                        prop_assert_eq!(msg_size, bad_size);
                    }
                    _ => {
                        let message = format!("Unexpected error type. {}", err);
                        return Err(TestCaseError::fail(message))
                    }
                }
            }
        }

        #[test]
        fn parse_eof(bad_bytes in proptest::collection::vec(proptest::num::u8::ANY, 0..16)) {
            let mut tmpfile = tempfile::tempfile()?;
            tmpfile.write(&bad_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match Message::parse_from_reader(&mut tmpfile) {
                Ok(_) => {
                    let message = format!("Message parsed correctly when it shouldn't.");
                    return Err(TestCaseError::fail(message));
                },
                Err(err) => match err {
                    MessageError::Io(err) => match err.kind() {
                        io::ErrorKind::UnexpectedEof => {},
                        _ => {
                            let message = format!("io::ErrorKind not UnexpectedEof. Got {}", err);
                            return Err(TestCaseError::fail(message));
                        }
                    }
                    _ => {
                        let message = format!("Unexpected error type. {}", err);
                        return Err(TestCaseError::fail(message));
                    }
                }
            }
        }

        #[test]
        fn parse_blank_bad_size(timestamp in 0..std::i64::MAX, bad_size in 1..std::i32::MAX) {
            // create temp file with the bad msg_size number
            let mut tmpfile = tempfile::tempfile()?;
            write_msg(&mut tmpfile, timestamp, BLANK_TYPE, bad_size, &[])?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match Message::parse_from_reader(&mut tmpfile) {
                Ok(_) => {
                    let message = format!("Message parsed correctly when it shouldn't.");
                    return Err(TestCaseError::fail(message));
                },
                Err(err) => match err {
                    MessageError::NonZeroSizeBlankMsg { msg_size } => {
                        prop_assert_eq!(msg_size, bad_size);
                    }
                    _ => {
                        let message = format!("Unexpected error type. {}", err);
                        return Err(TestCaseError::fail(message))
                    }
                }
            }
        }

        #[test]
        fn parse_blank(timestamp in 0..std::i64::MAX) {
            // create temp file with the bad msg_size number
            let mut tmpfile = tempfile::tempfile()?;
            write_msg(&mut tmpfile, timestamp, BLANK_TYPE, 0, &[])?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            let message = Message::parse_from_reader(&mut tmpfile)?;
            prop_assert_eq!(message.timestamp, timestamp);
            match message.msg_type {
                MessageType::Blank => {},
                _ => {
                    let message = format!("Mismatched message type. Got {:?}", message.msg_type);
                    return Err(TestCaseError::fail(message));
                }
            }
        }

        #[test]
        fn parse_unknown_eof(timestamp in 0..std::i64::MAX, (msg_bytes, msg_size) in random_msg_bytes_eof_strategy(100)) {
            let mut tmpfile = tempfile::tempfile()?;
            write_msg(&mut tmpfile, timestamp, UNKNOWN_TYPE, msg_size, &msg_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match Message::parse_from_reader(&mut tmpfile) {
                Ok(_) => {
                    let message = format!("Message parsed correctly when it shouldn't.");
                    return Err(TestCaseError::fail(message));
                },
                Err(err) => match err {
                    MessageError::Io(err) => match err.kind() {
                        io::ErrorKind::UnexpectedEof => {},
                        _ => {
                            let message = format!("io::ErrorKind not UnexpectedEof. Got {}", err);
                            return Err(TestCaseError::fail(message));
                        }
                    }
                    _ => {
                        let message = format!("Unexpected error type. {}", err);
                        return Err(TestCaseError::fail(message));
                    }
                }
            }
        }

        #[test]
        fn parse_unknown(timestamp in 0..std::i64::MAX, msg_bytes in random_msg_bytes_strategy(100)) {
            let mut tmpfile = tempfile::tempfile()?;
            write_msg(&mut tmpfile, timestamp, UNKNOWN_TYPE, msg_bytes.len() as i32, &msg_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            let message = Message::parse_from_reader(&mut tmpfile)?;
            prop_assert_eq!(message.timestamp, timestamp);
            match message.msg_type {
                MessageType::Unknown(parsed_msg_bytes) => prop_assert_eq!(parsed_msg_bytes, msg_bytes),
                _ => {
                    let message = format!("Mismatched message type. Got {:?}", message.msg_type);
                    return Err(TestCaseError::fail(message));
                }
            }
        }

        #[test]
        fn parse_unexpected_msg_type(timestamp in 0..std::i64::MAX, msg_type in prop::num::i32::ANY, msg_bytes in random_msg_bytes_strategy(100)) {
            prop_assume!(msg_type < 0 || msg_type > 4);

            let mut tmpfile = tempfile::tempfile()?;
            write_msg(&mut tmpfile, timestamp, UNKNOWN_TYPE, msg_bytes.len() as i32, &msg_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            let message = Message::parse_from_reader(&mut tmpfile)?;
            prop_assert_eq!(message.timestamp, timestamp);
            match message.msg_type {
                MessageType::Unknown(parsed_msg_bytes) => prop_assert_eq!(parsed_msg_bytes, msg_bytes),
                _ => {
                    let message = format!("Mismatched message type. Got {:?}", message.msg_type);
                    return Err(TestCaseError::fail(message));
                }
            }
        }


        #[test]
        fn parse_vision2010_eof(timestamp in 0..std::i64::MAX, (msg_bytes, msg_size) in random_msg_bytes_eof_strategy(100)) {
            let mut tmpfile = tempfile::tempfile()?;
            write_msg(&mut tmpfile, timestamp, VISION2010_TYPE, msg_size, &msg_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match Message::parse_from_reader(&mut tmpfile) {
                Ok(_) => {
                    let message = format!("Message parsed correctly when it shouldn't.");
                    return Err(TestCaseError::fail(message));
                },
                Err(err) => match err {
                    MessageError::Io(err) => match err.kind() {
                        io::ErrorKind::UnexpectedEof => {},
                        _ => {
                            let message = format!("io::ErrorKind not UnexpectedEof. Got {}", err);
                            return Err(TestCaseError::fail(message));
                        }
                    }
                    _ => {
                        let message = format!("Unexpected error type. {}", err);
                        return Err(TestCaseError::fail(message));
                    }
                }
            }
        }

        #[test]
        fn parse_vision2010(timestamp in 0..std::i64::MAX, msg_bytes in random_msg_bytes_strategy(100)) {
            let mut tmpfile = tempfile::tempfile()?;
            write_msg(&mut tmpfile, timestamp, VISION2010_TYPE, msg_bytes.len() as i32, &msg_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            let message = Message::parse_from_reader(&mut tmpfile)?;
            prop_assert_eq!(message.timestamp, timestamp);
            match message.msg_type {
                MessageType::Vision2010(parsed_msg_bytes) => prop_assert_eq!(parsed_msg_bytes, msg_bytes),
                _ => {
                    let message = format!("Mismatched message type. Got {:?}", message.msg_type);
                    return Err(TestCaseError::fail(message));
                }
            }
        }

        #[test]
        fn parse_refbox2013_eof(timestamp in 0..std::i64::MAX, (msg_bytes, msg_size) in random_msg_bytes_eof_strategy(100)) {
            let mut tmpfile = tempfile::tempfile()?;
            write_msg(&mut tmpfile, timestamp, REFBOX2013_TYPE, msg_size, &msg_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match Message::parse_from_reader(&mut tmpfile) {
                Ok(_) => {
                    let message = format!("Message parsed correctly when it shouldn't.");
                    return Err(TestCaseError::fail(message));
                },
                Err(err) => match err {
                    MessageError::Io(err) => match err.kind() {
                        io::ErrorKind::UnexpectedEof => {},
                        _ => {
                            let message = format!("io::ErrorKind not UnexpectedEof. Got {}", err);
                            return Err(TestCaseError::fail(message));
                        }
                    }
                    _ => {
                        let message = format!("Unexpected error type. {}", err);
                        return Err(TestCaseError::fail(message));
                    }
                }
            }
        }

        #[test]
        fn parse_refbox2013_bad_proto(timestamp in 0..std::i64::MAX, msg_bytes in random_msg_bytes_strategy(100)) {
            let mut tmpfile = tempfile::tempfile()?;
            write_msg(&mut tmpfile, timestamp, REFBOX2013_TYPE, msg_bytes.len() as i32, &msg_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match Message::parse_from_reader(&mut tmpfile) {
                Ok(_) => {
                    let message = format!("Message parsed correctly when it shouldn't.");
                    return Err(TestCaseError::fail(message));
                },
                Err(err) => match err {
                    MessageError::Protobuf(_) => {}
                    _ => {
                        let message = format!("Unexpected error type. {}", err);
                        return Err(TestCaseError::fail(message));
                    }
                }
            }
        }

        #[test]
        fn parse_refbox2013(timestamp in 0..std::i64::MAX, refbox_msg in refbox2013_strategy()) {
            let refbox_msg_bytes = refbox_msg.write_to_bytes()?;

            let mut tmpfile = tempfile::tempfile()?;
            write_msg(&mut tmpfile, timestamp, REFBOX2013_TYPE, refbox_msg_bytes.len() as i32, &refbox_msg_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            let message = Message::parse_from_reader(&mut tmpfile)?;
            prop_assert_eq!(message.timestamp, timestamp);
            match message.msg_type {
                MessageType::Refbox2013(parsed_refbox_msg) => prop_assert_eq!(parsed_refbox_msg, refbox_msg),
                _ => {
                    let message = format!("Mismatched message type. Got {:?}", message.msg_type);
                    return Err(TestCaseError::fail(message));
                }
            }
        }

        #[test]
        fn parse_vision2014_eof(timestamp in 0..std::i64::MAX, (msg_bytes, msg_size) in random_msg_bytes_eof_strategy(100)) {
            let mut tmpfile = tempfile::tempfile()?;
            write_msg(&mut tmpfile, timestamp, VISION2014_TYPE, msg_size, &msg_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match Message::parse_from_reader(&mut tmpfile) {
                Ok(_) => {
                    let message = format!("Message parsed correctly when it shouldn't.");
                    return Err(TestCaseError::fail(message));
                },
                Err(err) => match err {
                    MessageError::Io(err) => match err.kind() {
                        io::ErrorKind::UnexpectedEof => {},
                        _ => {
                            let message = format!("io::ErrorKind not UnexpectedEof. Got {}", err);
                            return Err(TestCaseError::fail(message));
                        }
                    }
                    _ => {
                        let message = format!("Unexpected error type. {}", err);
                        return Err(TestCaseError::fail(message));
                    }
                }
            }
        }

        #[test]
        #[ignore]
        fn parse_vision2014_bad_proto(timestamp in 0..std::i64::MAX, msg_bytes in random_msg_bytes_strategy(100)) {
            let mut tmpfile = tempfile::tempfile()?;
            write_msg(&mut tmpfile, timestamp, VISION2014_TYPE, msg_bytes.len() as i32, &msg_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            // TODO(dschwab): Message is successfully parsing even
            // when the size is 0. Not sure why this is the only test
            // with bad proto that is parsing correctly. I would
            // expect the required fields to cause parsing errors.
            match Message::parse_from_reader(&mut tmpfile) {
                Ok(_) => {
                    let message = format!("Message parsed correctly when it shouldn't.");
                    return Err(TestCaseError::fail(message));
                },
                Err(err) => match err {
                    MessageError::Protobuf(_) => {}
                    _ => {
                        let message = format!("Unexpected error type. {}", err);
                        return Err(TestCaseError::fail(message));
                    }
                }
            }
        }

        #[test]
        fn parse_vision2014(timestamp in 0..std::i64::MAX, vision_msg in vision2014_strategy()) {
            let vision_msg_bytes = vision_msg.write_to_bytes()?;

            let mut tmpfile = tempfile::tempfile()?;
            write_msg(&mut tmpfile, timestamp, VISION2014_TYPE, vision_msg_bytes.len() as i32, &vision_msg_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            let message = Message::parse_from_reader(&mut tmpfile)?;
            prop_assert_eq!(message.timestamp, timestamp);
            match message.msg_type {
                MessageType::Vision2014(parsed_vision_msg) => prop_assert_eq!(parsed_vision_msg, vision_msg),
                _ => {
                    let message = format!("Mismatched message type. Got {:?}", message.msg_type);
                    return Err(TestCaseError::fail(message));
                }
            }
        }
    }
}
