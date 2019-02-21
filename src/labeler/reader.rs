use super::*;
use crate::protos::log_labeler_data;
use byteorder::{BigEndian, ReadBytesExt};
use protobuf;
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

// This is necessary for the write_to_bytes method used below, but
// rust is generating a warning.
#[allow(unused_imports)]
use protobuf::Message;

#[derive(Debug, Fail)]
pub enum LabelerDataReaderError {
    #[fail(display = "{}", _0)]
    Io(#[fail(cause)] io::Error),
    #[fail(display = "{}", _0)]
    Protobuf(#[fail(cause)] protobuf::ProtobufError),
    #[fail(display = "invalid header string: {:?}", header)]
    InvalidHeader { header: Vec<u8> },
    #[fail(display = "unsupported log version: {}", version)]
    UnsupportedVersion { version: u32 },
}

impl From<io::Error> for LabelerDataReaderError {
    fn from(error: io::Error) -> Self {
        LabelerDataReaderError::Io(error)
    }
}

impl From<protobuf::ProtobufError> for LabelerDataReaderError {
    fn from(error: protobuf::ProtobufError) -> Self {
        LabelerDataReaderError::Protobuf(error)
    }
}

type LabelerDataReaderResult<T> = Result<T, LabelerDataReaderError>;

pub struct LabelerDataReader<T: Read + Seek> {
    reader: RefCell<BufReader<T>>,
    metadata: log_labeler_data::LabelerMetadata,
}

impl<T: Read + Seek> LabelerDataReader<T> {
    pub fn new(reader: T) -> LabelerDataReaderResult<LabelerDataReader<T>> {
        let mut reader = BufReader::new(reader);

        // read the file header
        let mut header = vec![0u8; LABELER_DATA_HEADER.len()];
        reader.read_exact(&mut header)?;
        if header != LABELER_DATA_HEADER {
            return Err(LabelerDataReaderError::InvalidHeader { header });
        }

        // check the version
        let version = reader.read_u32::<BigEndian>()?;
        if version != LABELER_DATA_VERSION {
            return Err(LabelerDataReaderError::UnsupportedVersion { version });
        }

        // should be the start of the first message (or metadata if no
        // messages in the data file)
        let curr_offset = reader.seek(SeekFrom::Current(0))?;

        // read the metadata
        let metadata_size_offset = -(std::mem::size_of::<u32>() as i64);
        reader.seek(SeekFrom::End(metadata_size_offset))?; // read a u64 from end
        let metadata_msg_size = reader.read_u32::<BigEndian>()? as usize;
        reader.seek(SeekFrom::End(
            metadata_size_offset - (metadata_msg_size as i64),
        ))?;

        let mut metadata_msg_bytes = vec![0u8; metadata_msg_size];
        reader.read_exact(&mut metadata_msg_bytes)?;
        let metadata =
            protobuf::parse_from_bytes::<log_labeler_data::LabelerMetadata>(&metadata_msg_bytes)?;

        // reset to current offset
        let new_offset = reader.seek(SeekFrom::Start(curr_offset))?;
        assert_eq!(new_offset, curr_offset); // double check successfully reset

        Ok(LabelerDataReader {
            reader: RefCell::new(reader),
            metadata,
        })
    }

    pub fn len(&self) -> usize {
        self.metadata.get_message_offsets().len()
    }

    pub fn is_empty(&self) -> bool {
        self.metadata.get_message_offsets().is_empty()
    }

    pub fn num_cameras(&self) -> u32 {
        self.metadata.get_num_cameras()
    }

    fn _read_message(&self) -> LabelerDataReaderResult<log_labeler_data::LabelerFrameGroup> {
        let mut reader = self.reader.borrow_mut();

        let msg_size = reader.read_u32::<BigEndian>()?;
        let mut msg_bytes = vec![0u8; msg_size as usize];
        reader.read_exact(&mut msg_bytes)?;

        Ok(protobuf::parse_from_bytes::<
            log_labeler_data::LabelerFrameGroup,
        >(&msg_bytes)?)
    }

    pub fn get(&self, index: usize) -> Option<log_labeler_data::LabelerFrameGroup> {
        if index >= self.metadata.get_message_offsets().len() {
            return None;
        }

        let offset = self.metadata.get_message_offsets()[index];
        let new_offset = self
            .reader
            .borrow_mut()
            .seek(SeekFrom::Start(offset))
            .ok()?;
        assert_eq!(offset, new_offset);

        Some(self._read_message().ok()?)
    }

    fn _read_messages(
        &self,
        message_offsets: &[u64],
    ) -> LabelerDataReaderResult<Vec<log_labeler_data::LabelerFrameGroup>> {
        let mut frame_groups = Vec::<log_labeler_data::LabelerFrameGroup>::new();

        for offset in message_offsets {
            let new_offset = self.reader.borrow_mut().seek(SeekFrom::Start(*offset))?;
            assert_eq!(*offset, new_offset);

            let frame_group = self._read_message()?;
            frame_groups.push(frame_group);
        }

        Ok(frame_groups)
    }

    pub fn get_range(
        &self,
        start: usize,
        end: usize,
    ) -> Option<Vec<log_labeler_data::LabelerFrameGroup>> {
        let offsets: Vec<u64> = self
            .metadata
            .get_message_offsets()
            .get(start..end)?
            .to_vec();

        Some(self._read_messages(&offsets).ok()?)
    }

    pub fn get_range_from(&self, start: usize) -> Option<Vec<log_labeler_data::LabelerFrameGroup>> {
        let offsets: Vec<u64> = self
            .metadata
            .get_message_offsets()
            .get(start..)?
            .to_vec();

        Some(self._read_messages(&offsets).ok()?)
    }
}

impl LabelerDataReader<File> {
    pub fn new_from_path(log_path: &Path) -> LabelerDataReaderResult<LabelerDataReader<File>> {
        let f = File::open(log_path)?;
        LabelerDataReader::new(f)
    }
}

pub struct LabelerDataReaderIntoIterator<T: Read + Seek> {
    reader: LabelerDataReader<T>,
    index: usize,
}

impl<T> Iterator for LabelerDataReaderIntoIterator<T>
where
    T: Read + Seek,
{
    type Item = log_labeler_data::LabelerFrameGroup;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.reader.get(self.index)?;
        self.index += 1;

        Some(item)
    }
}

impl<T> IntoIterator for LabelerDataReader<T>
where
    T: Read + Seek,
{
    type Item = <LabelerDataReaderIntoIterator<T> as Iterator>::Item;
    type IntoIter = LabelerDataReaderIntoIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            reader: self,
            index: 0,
        }
    }
}

pub struct LabelerDataReaderIterator<'a, T: Read + Seek> {
    reader: &'a LabelerDataReader<T>,
    index: usize,
}

impl<'a, T> Iterator for LabelerDataReaderIterator<'a, T>
where
    T: Read + Seek,
{
    type Item = log_labeler_data::LabelerFrameGroup;
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.reader.get(self.index)?;
        self.index += 1;

        Some(item)
    }
}

impl<'a, T> IntoIterator for &'a LabelerDataReader<T>
where
    T: Read + Seek,
{
    type Item = <LabelerDataReaderIterator<'a, T> as Iterator>::Item;
    type IntoIter = LabelerDataReaderIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        LabelerDataReaderIterator {
            reader: &self,
            index: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protos::log_labeler_data;
    use crate::test_utils::labeler as test_utils_labeler;
    use byteorder::{BigEndian, WriteBytesExt};
    use proptest::prelude::*;
    use std::io::{Cursor, Seek, SeekFrom, Write};

    impl From<LabelerDataReaderError> for TestCaseError {
        fn from(error: LabelerDataReaderError) -> Self {
            TestCaseError::fail(format!("{}", error))
        }
    }

    fn write_header<T: Write>(
        writer: &mut T,
        header: &[u8],
        version: u32,
    ) -> LabelerDataReaderResult<()> {
        writer.write_all(header)?;
        writer.write_u32::<BigEndian>(version)?;

        Ok(())
    }

    fn write_msgs<T: Write + Seek>(
        writer: &mut T,
        frame_group_msgs: &Vec<log_labeler_data::LabelerFrameGroup>,
    ) -> LabelerDataReaderResult<log_labeler_data::LabelerMetadata> {
        let mut num_cameras = 0u32;
        let mut message_offsets = Vec::<u64>::new();

        for frame_group_msg in frame_group_msgs {
            for frame in &frame_group_msg.frames {
                if frame.has_vision_frame() {
                    num_cameras = std::cmp::max(
                        num_cameras,
                        frame.get_vision_frame().get_detection().get_camera_id(),
                    );
                }
            }

            let curr_offset = writer.seek(SeekFrom::Current(0))?;
            message_offsets.push(curr_offset);

            let frame_msg_bytes = frame_group_msg.write_to_bytes()?;

            writer.write_u32::<BigEndian>(frame_msg_bytes.len() as u32)?;
            writer.write_all(&frame_msg_bytes)?;
        }

        let mut labeler_metadata = log_labeler_data::LabelerMetadata::new();
        labeler_metadata.set_num_cameras(num_cameras);
        labeler_metadata.set_message_offsets(message_offsets);
        write_metadata(writer, &labeler_metadata)?;

        Ok(labeler_metadata)
    }

    fn write_metadata<T: Write>(
        writer: &mut T,
        metadata: &log_labeler_data::LabelerMetadata,
    ) -> LabelerDataReaderResult<()> {
        let metadata_bytes = metadata.write_to_bytes()?;
        writer.write_all(&metadata_bytes)?;
        writer.write_u32::<BigEndian>(metadata_bytes.len() as u32)?;

        Ok(())
    }

    proptest! {
        #[test]
        fn new_bad_header(bad_header in proptest::collection::vec(proptest::num::u8::ANY, LABELER_DATA_HEADER.len())) {
            prop_assume!(bad_header != LABELER_DATA_HEADER);

            let mut buffer = Vec::<u8>::new();
            let mut writer = Cursor::new(&mut buffer);
            write_header(&mut writer, &bad_header, LABELER_DATA_VERSION)?;
            write_metadata(&mut writer, &log_labeler_data::LabelerMetadata::new())?;
            drop(writer);

            let reader = Cursor::new(buffer.as_mut_slice());
            match LabelerDataReader::new(reader) {
                Err(error) => match error {
                    LabelerDataReaderError::InvalidHeader { header } => prop_assert_eq!(header, bad_header),
                    _ => return Err(TestCaseError::fail(format!("Unexpected error: {}", error))),
                }
                Ok(_) => return Err(TestCaseError::fail("Constructed data reader when it should have failed.")),
            };
        }

        #[test]
        fn new_bad_version(bad_version in proptest::num::u32::ANY) {
            prop_assume!(bad_version != LABELER_DATA_VERSION);

            let mut buffer = Vec::<u8>::new();
            let mut writer = Cursor::new(&mut buffer);
            write_header(&mut writer, &LABELER_DATA_HEADER, bad_version)?;
            write_metadata(&mut writer, &log_labeler_data::LabelerMetadata::new())?;
            drop(writer);

            let reader = Cursor::new(buffer.as_mut_slice());
            match LabelerDataReader::new(reader) {
                Err(error) => match error {
                    LabelerDataReaderError::UnsupportedVersion { version } => prop_assert_eq!(version, bad_version),
                    _ => return Err(TestCaseError::fail(format!("Unexpected error: {}", error))),
                }
                Ok(_) => return Err(TestCaseError::fail("Constructed data reader when it should have failed.")),
            };
        }

        #[test]
        fn new_bad_metadata_size(bad_metadata_size in 1..std::u32::MAX) {
            let mut buffer = Vec::<u8>::new();
            let mut writer = Cursor::new(&mut buffer);
            write_header(&mut writer, &LABELER_DATA_HEADER, LABELER_DATA_VERSION)?;
            writer.write_u32::<BigEndian>(bad_metadata_size)?;
            drop(writer);

            // either get an io or a protobuf error,
            // depending on the size and how the bytes are
            // interpreted.
            let reader = Cursor::new(buffer.as_mut_slice());
            match LabelerDataReader::new(reader) {
                Err(error) => match error {
                    LabelerDataReaderError::Io(_) | LabelerDataReaderError::Protobuf(_) => {},
                    _ => return Err(TestCaseError::fail(format!("Unexpected error: {}", error))),
                },
                Ok(_) => return Err(TestCaseError::fail("Constructed data reader when it should have failed.")),
            };
        }

        #[test]
        fn new_random_frames(
            random_frame_group_msgs in proptest::collection::vec(
                test_utils_labeler::random_no_ref_frame_group_msg_strategy(1, 10), 1..10)) {
            let mut buffer = Vec::<u8>::new();
            let mut writer = Cursor::new(&mut buffer);
            write_header(&mut writer, &LABELER_DATA_HEADER, LABELER_DATA_VERSION)?;
            let metadata = write_msgs(&mut writer, &random_frame_group_msgs)?;
            drop(writer);

            let reader = LabelerDataReader::new(Cursor::new(buffer.as_mut_slice()))?;

            prop_assert_eq!(reader.num_cameras(), metadata.get_num_cameras());
            prop_assert_eq!(reader.len(), metadata.get_message_offsets().len());
        }

        #[test]
        fn get_random_frames(
            random_frames_and_access_pattern in
                test_utils_labeler::random_frames_and_access_pattern_strategy(1, 10, 2, 10)) {
            let (random_frame_group_msgs, message_indexes) = random_frames_and_access_pattern;

            let mut buffer = Vec::<u8>::new();
            let mut writer = Cursor::new(&mut buffer);
            write_header(&mut writer, &LABELER_DATA_HEADER, LABELER_DATA_VERSION)?;
            let metadata = write_msgs(&mut writer, &random_frame_group_msgs)?;
            drop(writer);

            let mut reader = LabelerDataReader::new(Cursor::new(buffer.as_mut_slice()))?;

            prop_assert_eq!(reader.num_cameras(), metadata.get_num_cameras());
            prop_assert_eq!(reader.len(), metadata.get_message_offsets().len());

            for index in message_indexes {
                match reader.get(index) {
                    Some(ref frame_group) => prop_assert_eq!(frame_group, &random_frame_group_msgs[index]),
                    None => return Err(TestCaseError::fail(format!("Failed to get message at index {}", index))),
                }
            }
        }

        #[test]
        fn get_range(random_frames_and_ranges in test_utils_labeler::random_frames_and_ranges_strategy(1, 10, 2, 10)) {
            let (random_frame_group_msgs, range_indexes) = random_frames_and_ranges;

            let mut buffer = Vec::<u8>::new();
            let mut writer = Cursor::new(&mut buffer);
            write_header(&mut writer, &LABELER_DATA_HEADER, LABELER_DATA_VERSION)?;
            let metadata = write_msgs(&mut writer, &random_frame_group_msgs)?;
            drop(writer);

            let mut reader = LabelerDataReader::new(Cursor::new(buffer.as_mut_slice()))?;

            prop_assert_eq!(reader.num_cameras(), metadata.get_num_cameras());
            prop_assert_eq!(reader.len(), metadata.get_message_offsets().len());

            for range_index in range_indexes {
                match reader.get_range(range_index.0, range_index.1) {
                    Some(frame_groups) => {
                        prop_assert_eq!(frame_groups.len(), range_index.1-range_index.0);
                        for (frame_group, index) in frame_groups.iter().zip(range_index.0..range_index.1) {
                            prop_assert_eq!(frame_group, &random_frame_group_msgs[index]);
                        }
                    },
                    None => return Err(TestCaseError::fail(format!("Failed to get message with range {:?}", range_index))),
                }
            }
        }

        #[test]
        fn get_range_from(random_frames_and_ranges in test_utils_labeler::random_frames_and_ranges_strategy(1, 10, 2, 10)) {
            let (random_frame_group_msgs, range_indexes) = random_frames_and_ranges;

            let mut buffer = Vec::<u8>::new();
            let mut writer = Cursor::new(&mut buffer);
            write_header(&mut writer, &LABELER_DATA_HEADER, LABELER_DATA_VERSION)?;
            let metadata = write_msgs(&mut writer, &random_frame_group_msgs)?;
            drop(writer);

            let reader = LabelerDataReader::new(Cursor::new(buffer.as_mut_slice()))?;

            prop_assert_eq!(reader.num_cameras(), metadata.get_num_cameras());
            prop_assert_eq!(reader.len(), metadata.get_message_offsets().len());

            for range_index in range_indexes {
                match reader.get_range_from(range_index.0) {
                    Some(frame_groups) => {
                        prop_assert_eq!(frame_groups.len(), reader.len()-range_index.0);
                        for (frame_group, index) in frame_groups.iter().zip(range_index.0..reader.len()) {
                            prop_assert_eq!(frame_group, &random_frame_group_msgs[index]);
                        }
                    },
                    None => return Err(TestCaseError::fail(format!("Failed to get message with range {:?}", range_index))),
                }
            }
        }

        #[test]
        fn into_iterate_random_frames(
            random_frame_group_msgs in proptest::collection::vec(
                test_utils_labeler::random_no_ref_frame_group_msg_strategy(1, 10), 1..10)) {
            let mut buffer = Vec::<u8>::new();
            let mut writer = Cursor::new(&mut buffer);
            write_header(&mut writer, &LABELER_DATA_HEADER, LABELER_DATA_VERSION)?;
            let metadata = write_msgs(&mut writer, &random_frame_group_msgs)?;
            drop(writer);

            let reader = LabelerDataReader::new(Cursor::new(buffer.as_mut_slice()))?;

            prop_assert_eq!(reader.num_cameras(), metadata.get_num_cameras());
            prop_assert_eq!(reader.len(), metadata.get_message_offsets().len());

            for (read_frame_msg, frame_msg) in reader.into_iter().zip(random_frame_group_msgs) {
                prop_assert_eq!(read_frame_msg, frame_msg);
            }
        }

        #[test]
        fn iterate_random_frames(
            random_frame_group_msgs in proptest::collection::vec(
                test_utils_labeler::random_no_ref_frame_group_msg_strategy(1, 10), 1..10)) {
            let mut buffer = Vec::<u8>::new();
            let mut writer = Cursor::new(&mut buffer);
            write_header(&mut writer, &LABELER_DATA_HEADER, LABELER_DATA_VERSION)?;
            let metadata = write_msgs(&mut writer, &random_frame_group_msgs)?;
            drop(writer);

            let reader = LabelerDataReader::new(Cursor::new(buffer.as_mut_slice()))?;

            prop_assert_eq!(reader.num_cameras(), metadata.get_num_cameras());
            prop_assert_eq!(reader.len(), metadata.get_message_offsets().len());
            prop_assert!(!reader.is_empty());

            for (read_frame_msg, frame_msg) in (&reader).into_iter().zip(&random_frame_group_msgs) {
                prop_assert_eq!(read_frame_msg, frame_msg.clone());
            }

            // should be able to compile without move errors
            for (read_frame_msg, frame_msg) in (&reader).into_iter().zip(&random_frame_group_msgs) {
                prop_assert_eq!(read_frame_msg, frame_msg.clone());
            }
        }

    }

    #[test]
    fn new_blank() {
        let mut buffer = Vec::<u8>::new();
        let mut writer = Cursor::new(&mut buffer);
        write_header(&mut writer, &LABELER_DATA_HEADER, LABELER_DATA_VERSION).unwrap();
        write_metadata(&mut writer, &log_labeler_data::LabelerMetadata::new()).unwrap();
        drop(writer);

        let reader = Cursor::new(buffer.as_mut_slice());
        let reader = LabelerDataReader::new(reader).unwrap();

        assert_eq!(reader.len(), 0);
        assert!(reader.is_empty());
        assert_eq!(reader.num_cameras(), 0);
    }

    #[test]
    fn new_from_path_blank() {
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        write_header(&mut tmpfile, &LABELER_DATA_HEADER, LABELER_DATA_VERSION).unwrap();
        write_metadata(&mut tmpfile, &log_labeler_data::LabelerMetadata::new()).unwrap();
        tmpfile.seek(io::SeekFrom::Start(0)).unwrap();

        let reader = LabelerDataReader::new_from_path(tmpfile.path()).unwrap();

        assert_eq!(reader.len(), 0);
        assert!(reader.is_empty());
        assert_eq!(reader.num_cameras(), 0);
    }

}
