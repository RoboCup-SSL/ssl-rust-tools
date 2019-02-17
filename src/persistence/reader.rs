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
}

impl From<io::Error> for LogReaderError {
    fn from(error: io::Error) -> Self {
        LogReaderError::Io(error)
    }
}

#[derive(Debug)]
pub struct LogReader<T: Read + Seek> {
    reader: BufReader<T>,
}

// TODO(dschwab): Is there a better way of doing the individual
// character conversions without requiring lazy_static?
const EXPECTED_HEADER: [u8; 12] = [
    'S' as u8, 'S' as u8, 'L' as u8, '_' as u8, 'L' as u8, 'O' as u8, 'G' as u8, '_' as u8,
    'F' as u8, 'I' as u8, 'L' as u8, 'E' as u8,
];

const SUPPORTED_VERSION: i32 = 1;

impl<T: Read + Seek> LogReader<T> {
    pub fn new(mut reader: T) -> Result<LogReader<T>, LogReaderError> {
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
        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes)?;
        let version: i32 = unsafe { std::mem::transmute(version_bytes) };
        if version != SUPPORTED_VERSION {
            return Err(LogReaderError::UnsupportedVersion { version });
        }

        Ok(LogReader { reader })
    }
}

impl LogReader<File> {
    pub fn new_from_path(log_path: &Path) -> Result<LogReader<File>, LogReaderError> {
        let f = File::open(log_path)?;
        LogReader::new(f)
    }
}

#[cfg(test)]
mod tests {
    extern crate proptest;
    extern crate tempfile;

    use super::*;
    use proptest::prelude::*;
    use std::io::Write;

    proptest! {
        #[test]
        fn new_from_path_bad_header(bad_header in prop::array::uniform12(prop::bits::u8::ANY)) {
            prop_assume!(bad_header != EXPECTED_HEADER);

            // create the temp file with the bad header
            let mut tmpfile = tempfile::NamedTempFile::new()?;
            tmpfile.write(&bad_header)?;
            let version_bytes: [u8; 4] = unsafe { std::mem::transmute(1i32) };
            tmpfile.write(&version_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match LogReader::new_from_path(tmpfile.path()).unwrap_err() {
                LogReaderError::InvalidHeader { header } => {
                    prop_assert_eq!(header, bad_header);
                }
                e @ _ => {
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
            tmpfile.write(&bad_header)?;
            let version_bytes: [u8; 4] = unsafe { std::mem::transmute(1i32) };
            tmpfile.write(&version_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match LogReader::new(tmpfile).unwrap_err() {
                LogReaderError::InvalidHeader { header } => {
                    prop_assert_eq!(header, bad_header);
                }
                e @ _ => {
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
            tmpfile.write(&EXPECTED_HEADER)?;
            let version_bytes: [u8; 4] = unsafe { std::mem::transmute(bad_version) };
            tmpfile.write(&version_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match LogReader::new_from_path(tmpfile.path()).unwrap_err() {
                LogReaderError::UnsupportedVersion { version } => {
                    prop_assert_eq!(version, bad_version);
                }
                e @ _ => {
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
            tmpfile.write(&EXPECTED_HEADER)?;
            let version_bytes: [u8; 4] = unsafe { std::mem::transmute(bad_version) };
            tmpfile.write(&version_bytes)?;
            tmpfile.seek(io::SeekFrom::Start(0))?;

            match LogReader::new(tmpfile).unwrap_err() {
                LogReaderError::UnsupportedVersion { version } => {
                    prop_assert_eq!(version, bad_version);
                }
                e @ _ => {
                    let message = format!("Unexpected error type. {}", e);
                    return Err(TestCaseError::fail(message));
                },
            };
        }
    }
}
