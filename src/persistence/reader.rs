use super::message;
use byteorder::{BigEndian, ReadBytesExt};
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;
use std::str;

#[derive(Debug, Fail)]
pub enum LogReaderError {
    #[fail(display = "{}", _0)]
    Io(#[fail(cause)] io::Error),
    #[fail(display = "invalid header string: {:?}", header)]
    InvalidHeader { header: [u8; 12] },
    #[fail(display = "unsupported log version: {}", version)]
    UnsupportedVersion { version: i32 },
    #[fail(display = "{}", _0)]
    Message(#[fail(cause)] message::MessageError),
}

impl From<io::Error> for LogReaderError {
    fn from(error: io::Error) -> Self {
        LogReaderError::Io(error)
    }
}

impl From<message::MessageError> for LogReaderError {
    fn from(error: message::MessageError) -> Self {
        match error {
            message::MessageError::Io(e) => LogReaderError::Io(e),
            e => LogReaderError::Message(e),
        }
    }
}

#[derive(Debug)]
pub struct LogReader<T: Read> {
    reader: BufReader<T>,
}

const EXPECTED_HEADER: [u8; 12] = [
    b'S', b'S', b'L', b'_', b'L', b'O', b'G', b'_', b'F', b'I', b'L', b'E',
];

const SUPPORTED_VERSION: i32 = 1;

impl<T: Read> LogReader<T> {
    pub fn new(reader: T) -> Result<LogReader<T>, LogReaderError> {
        let mut reader = BufReader::new(reader);

        // read first 12 bytes, which should equal "SSL_LOG_FILE"
        let mut header = [0u8; 12];
        reader.read_exact(&mut header)?;
        if header != EXPECTED_HEADER {
            // at least one of the bytes doesn't match
            // so return an error
            return Err(LogReaderError::InvalidHeader { header });
        }

        // read a 32 bit integer, and check that the version matches
        // the expected log version
        let version = reader.read_i32::<BigEndian>()?;
        if version != SUPPORTED_VERSION {
            return Err(LogReaderError::UnsupportedVersion { version });
        }

        Ok(LogReader { reader })
    }

    pub fn read_message(&mut self) -> Result<message::Message, LogReaderError> {
        Ok(message::Message::parse_from_reader(&mut self.reader)?)
    }

    pub fn inner(&self) -> &BufReader<T> {
        &self.reader
    }

    pub fn inner_mut(&mut self) -> &mut BufReader<T> {
        &mut self.reader
    }
}

impl LogReader<File> {
    pub fn new_from_path(log_path: &Path) -> Result<LogReader<File>, LogReaderError> {
        let f = File::open(log_path)?;
        LogReader::new(f)
    }
}

impl<T> Iterator for LogReader<T>
where
    T: Read,
{
    type Item = Result<message::Message, LogReaderError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_message() {
            Err(error) => match error {
                LogReaderError::Io(ref err) => match err.kind() {
                    io::ErrorKind::UnexpectedEof => None,
                    _ => Some(Err(error)),
                },
                _ => Some(Err(error)),
            },
            o @ Ok(_) => Some(o),
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate proptest;
    extern crate tempfile;

    use super::*;
    use crate::test_utils::reader::*;
    use byteorder::{BigEndian, WriteBytesExt};
    use proptest::prelude::*;
    use std::io::Write;

    impl From<LogReaderError> for TestCaseError {
        fn from(error: LogReaderError) -> Self {
            TestCaseError::fail(format!("{}", error))
        }
    }

    fn write_random_log(messages: &[message::Message]) -> Result<Vec<u8>, message::MessageError> {
        // TODO(dschwab): probably better to pre-calculate size and
        // reserve all the memory at once.
        let mut output = Vec::<u8>::new();

        // write the log header
        output.extend(EXPECTED_HEADER.iter().cloned());
        output.write_i32::<BigEndian>(SUPPORTED_VERSION)?;

        // write all the messages
        for message in messages {
            message.write_to_vec(&mut output)?;
        }

        Ok(output)
    }

    proptest! {
        #[test]
        fn new_from_path_bad_header(bad_header in prop::array::uniform12(prop::bits::u8::ANY)) {
            prop_assume!(bad_header != EXPECTED_HEADER);

            // create the temp file with the bad header
            let mut tmpfile = tempfile::NamedTempFile::new()?;
            tmpfile.write_all(&bad_header)?;
            tmpfile.write_i32::<BigEndian>(SUPPORTED_VERSION)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match LogReader::new_from_path(tmpfile.path()).unwrap_err() {
                LogReaderError::InvalidHeader { header } => {
                    prop_assert_eq!(header, bad_header);
                }
                e => {
                    let message = format!("Unexpected error type. {}", e);
                    return Err(TestCaseError::fail(message));
                },
            };
        }

        #[test]
        fn new_bad_header(bad_header in prop::array::uniform12(prop::bits::u8::ANY)) {
            prop_assume!(bad_header != EXPECTED_HEADER);

            // create the temp file with the bad header
            let mut tmpfile = tempfile::tempfile()?;
            tmpfile.write_all(&bad_header)?;
            tmpfile.write_i32::<BigEndian>(SUPPORTED_VERSION)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match LogReader::new(tmpfile).unwrap_err() {
                LogReaderError::InvalidHeader { header } => {
                    prop_assert_eq!(header, bad_header);
                }
                e => {
                    let message = format!("Unexpected error type. {}", e);
                    return Err(TestCaseError::fail(message));
                },
            };
        }

        #[test]
        fn new_from_path_bad_version(bad_version in proptest::num::i32::ANY) {
            prop_assume!(bad_version != SUPPORTED_VERSION);

            // create temp file with the bad version number
            let mut tmpfile = tempfile::NamedTempFile::new()?;
            tmpfile.write_all(&EXPECTED_HEADER)?;
            tmpfile.write_i32::<BigEndian>(bad_version)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match LogReader::new_from_path(tmpfile.path()).unwrap_err() {
                LogReaderError::UnsupportedVersion { version } => {
                    prop_assert_eq!(version, bad_version);
                }
                e => {
                    let message = format!("Unexpected error type. {}", e);
                    return Err(TestCaseError::fail(message));
                },
            };
        }

        #[test]
        fn new_bad_version(bad_version in proptest::num::i32::ANY) {
            prop_assume!(bad_version != SUPPORTED_VERSION);

            // create temp file with the bad version number
            let mut tmpfile = tempfile::tempfile()?;
            tmpfile.write_all(&EXPECTED_HEADER)?;
            tmpfile.write_i32::<BigEndian>(bad_version)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match LogReader::new(tmpfile).unwrap_err() {
                LogReaderError::UnsupportedVersion { version } => {
                    prop_assert_eq!(version, bad_version);
                }
                e => {
                    let message = format!("Unexpected error type. {}", e);
                    return Err(TestCaseError::fail(message));
                },
            };
        }

        #[test]
        fn read_messages(messages in random_messages(1, 10)) {
            let log_bytes = write_random_log(&messages)?;

            let mut reader = LogReader::new(log_bytes.as_slice())?;
            for expected_msg in &messages {
                let message = reader.read_message()?;
                prop_assert_eq!(&message, expected_msg);
            }
        }

        #[test]
        fn iterate_messages(messages in random_messages(1, 10)) {
            let log_bytes = write_random_log(&messages)?;

            let reader = LogReader::new(log_bytes.as_slice())?;
            for (expected_msg, message) in messages.iter().zip(reader) {
                match message {
                    Ok(message) => prop_assert_eq!(&message, expected_msg),
                    Err(e) => return Err(e.into())
                }
            }
        }
    }

    #[test]
    fn read_messages_empty() {
        let messages = Vec::<message::Message>::new();
        let log_bytes = write_random_log(&messages).unwrap();

        let mut reader = LogReader::new(log_bytes.as_slice()).unwrap();

        match reader.read_message().unwrap_err() {
            LogReaderError::Io(e) => match e.kind() {
                io::ErrorKind::UnexpectedEof => {}
                _ => panic!("Unexpected io error kind {:?}", e),
            },
            e => panic!("Unexpectged error type {}", e),
        };
    }

}
