use super::*;
use crate::persistence::message::{Message, MessageType};
use crate::protos::log_labeler_data;
use crate::protos::messages_robocup_ssl_referee::SSL_Referee_Stage;
use byteorder::{BigEndian, WriteBytesExt};
use protobuf;
use protobuf::Message as ProtobufMessage;
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

#[derive(Debug, Fail)]
pub enum LabelerDataWriterError {
    #[fail(display = "{}", _0)]
    Io(#[fail(cause)] io::Error),
    #[fail(display = "{}", _0)]
    Protobuf(#[fail(cause)] protobuf::ProtobufError),
}

impl From<io::Error> for LabelerDataWriterError {
    fn from(error: io::Error) -> Self {
        LabelerDataWriterError::Io(error)
    }
}

impl From<protobuf::ProtobufError> for LabelerDataWriterError {
    fn from(error: protobuf::ProtobufError) -> Self {
        LabelerDataWriterError::Protobuf(error)
    }
}

type LabelerDataWriterResult<T> = Result<T, LabelerDataWriterError>;

pub struct LabelerDataWriter<T: Write + Seek> {
    writer: BufWriter<T>,
    num_cameras: u32,
    // option allows moving out of self during drop, which prevents a
    // full copy of the vector being made
    message_offsets: Option<Vec<u64>>,
    // book-keeping for building up current LabelerData message
    curr_stage: Option<SSL_Referee_Stage>,
    // option allows taking the internal vector without copying
    curr_frames: Option<Vec<log_labeler_data::LabelerFrame>>,
    curr_cam_set: HashSet<u32>,
}

impl<T: Write + Seek> LabelerDataWriter<T> {
    pub fn new(writer: T) -> LabelerDataWriterResult<LabelerDataWriter<T>> {
        let mut writer = BufWriter::new(writer);

        // write the file header
        writer.write_all(&LABELER_DATA_HEADER)?;

        // write the file version
        writer.write_u32::<BigEndian>(LABELER_DATA_VERSION)?;

        Ok(LabelerDataWriter {
            writer,
            num_cameras: 0,
            message_offsets: Some(Vec::<u64>::new()),
            curr_stage: None,
            curr_frames: Some(Vec::new()),
            curr_cam_set: HashSet::new(),
        })
    }

    pub fn inner(&self) -> &BufWriter<T> {
        &self.writer
    }

    pub fn inner_mut(&mut self) -> &mut BufWriter<T> {
        &mut self.writer
    }

    pub fn add_msg(&mut self, message: Message) -> LabelerDataWriterResult<()> {
        match message.msg_type {
            MessageType::Refbox2013(ref_msg) => {
                let new_stage = Some(ref_msg.get_stage());
                if new_stage != self.curr_stage {
                    self.write_frame()?;
                }

                // update the current stage
                self.curr_stage = Some(ref_msg.get_stage());

                if is_running_stage(self.curr_stage) {
                    let mut frame = log_labeler_data::LabelerFrame::new();
                    frame.set_timestamp(message.timestamp as u64);
                    frame.set_referee_frame(ref_msg);
                    self.curr_frames.as_mut().unwrap().push(frame);
                }
            }
            MessageType::Vision2014(vision_msg) => {
                if is_running_stage(self.curr_stage) {
                    let cam_id = vision_msg.get_detection().get_camera_id();
                    self.num_cameras = std::cmp::max(self.num_cameras, cam_id);
                    if self.curr_cam_set.contains(&cam_id) {
                        self.write_frame()?;
                    }

                    // add the current cam id to the set
                    let not_contained = self.curr_cam_set.insert(cam_id);
                    assert!(not_contained);

                    let mut frame = log_labeler_data::LabelerFrame::new();
                    frame.set_timestamp(message.timestamp as u64);
                    frame.set_vision_frame(vision_msg);
                    self.curr_frames.as_mut().unwrap().push(frame);
                }
            }
            _ => {}
        };

        Ok(())
    }

    fn write_frame(&mut self) -> LabelerDataWriterResult<()> {
        // only write if there is same data in the current frame
        if !self.curr_frames.as_ref().unwrap_or(&vec![]).is_empty() {
            let curr_offset = self.writer.seek(SeekFrom::Current(0))?;
            self.message_offsets.as_mut().unwrap().push(curr_offset);

            let mut frame_group = log_labeler_data::LabelerFrameGroup::new();
            frame_group.set_frames(self.curr_frames.take().unwrap().into());

            let msg_bytes = frame_group.write_to_bytes()?;
            self.writer.write_u32::<BigEndian>(msg_bytes.len() as u32)?;
            self.writer.write_all(&msg_bytes)?;
        }

        // reset the book-keeping parts of the writer struct
        self.curr_frames = Some(Vec::new());
        self.curr_cam_set = HashSet::new();

        Ok(())
    }
}

impl LabelerDataWriter<File> {
    pub fn new_from_path(log_path: &Path) -> LabelerDataWriterResult<LabelerDataWriter<File>> {
        let f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(log_path)?;
        LabelerDataWriter::new(f)
    }
}

impl<T: Write + Seek> Drop for LabelerDataWriter<T> {
    // write the metadata message to the end of the file
    fn drop(&mut self) {
        // write frame in case there is any lingering data
        self.write_frame().unwrap();

        let mut metadata = log_labeler_data::LabelerMetadata::new();
        metadata.set_num_cameras(self.num_cameras);
        let message_offsets = self.message_offsets.take().unwrap_or(vec![]);
        metadata.set_message_offsets(message_offsets);

        let metadata_bytes = metadata.write_to_bytes().unwrap();
        self.writer.write_all(&metadata_bytes).unwrap();
        self.writer
            .write_u32::<BigEndian>(metadata_bytes.len() as u32)
            .unwrap();
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
        None => false,
    }
}

#[cfg(test)]
mod tests {
    extern crate proptest;
    extern crate tempfile;

    use super::*;
    use crate::protos::log_labeler_data;
    use crate::test_utils::labeler as test_utils_labeler;
    use crate::test_utils::message as test_utils_message;
    use byteorder::{BigEndian, ReadBytesExt};
    use proptest::prelude::*;
    use std::io::{Cursor, Read, Seek, SeekFrom};

    impl From<LabelerDataWriterError> for TestCaseError {
        fn from(error: LabelerDataWriterError) -> Self {
            TestCaseError::fail(format!("{}", error))
        }
    }

    fn check_header<T: Read>(mut reader: T) -> LabelerDataWriterResult<()> {
        // check header
        let mut header = [0u8; 16];
        reader.read_exact(&mut header)?;
        assert_eq!(header, LABELER_DATA_HEADER);

        // check the version
        let version = reader.read_u32::<BigEndian>()?;
        assert_eq!(version, LABELER_DATA_VERSION);

        Ok(())
    }

    fn get_metadata<T: Read + Seek>(
        mut reader: T,
    ) -> LabelerDataWriterResult<log_labeler_data::LabelerMetadata> {
        let metadata_size_offset = -(std::mem::size_of::<u32>() as i64);
        reader.seek(SeekFrom::End(metadata_size_offset))?; // read a u64 from end
        let metadata_msg_size = reader.read_u32::<BigEndian>()? as usize;
        reader.seek(SeekFrom::End(
            metadata_size_offset - (metadata_msg_size as i64),
        ))?;

        let mut metadata_msg_bytes = vec![0u8; metadata_msg_size];
        reader.read_exact(&mut metadata_msg_bytes)?;
        Ok(protobuf::parse_from_bytes::<
            log_labeler_data::LabelerMetadata,
        >(&metadata_msg_bytes)?)
    }

    fn get_frame_at<T: Read + Seek>(
        mut reader: T,
        offset: u64,
    ) -> LabelerDataWriterResult<log_labeler_data::LabelerFrameGroup> {
        reader.seek(SeekFrom::Start(offset))?;
        let frame_size = reader.read_u32::<BigEndian>()? as usize;
        let mut frame_bytes = vec![0u8; frame_size];
        reader.read_exact(&mut frame_bytes)?;
        Ok(protobuf::parse_from_bytes::<
            log_labeler_data::LabelerFrameGroup,
        >(&frame_bytes)?)
    }

    #[test]
    fn write_empty_to_buffer() {
        let mut buffer = Vec::<u8>::new();
        let writer = Cursor::new(&mut buffer);
        LabelerDataWriter::new(writer).unwrap();

        // check header
        check_header(Cursor::new(buffer.as_mut_slice())).unwrap();

        // Read the metadata message
        let metadata = get_metadata(Cursor::new(buffer.as_mut_slice())).unwrap();

        // check that it matches expected values
        assert_eq!(metadata.get_num_cameras(), 0);
        assert_eq!(metadata.get_message_offsets().len(), 0);
    }

    #[test]
    fn write_empty_to_file() {
        // create temp file
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();

        // create blank labeler writer
        LabelerDataWriter::new_from_path(tmpfile.path()).unwrap();

        // check header
        tmpfile.seek(io::SeekFrom::Start(0)).unwrap();
        check_header(&tmpfile).unwrap();

        // read metadata message
        let metadata = get_metadata(&tmpfile).unwrap();

        // check that it matches expected values
        assert_eq!(metadata.get_num_cameras(), 0);
        assert_eq!(metadata.get_message_offsets().len(), 0);
    }

    proptest! {

        #[test]
        fn skip_blank_msg(blank_msg in test_utils_message::random_blank_msg_strategy()) {
            let mut buffer = Vec::<u8>::new();
            let writer = Cursor::new(&mut buffer);

            let mut writer = LabelerDataWriter::new(writer)?;
            writer.add_msg(blank_msg)?;
            drop(writer);

            // check header
            check_header(Cursor::new(buffer.as_mut_slice()))?;

            // Read the metadata message
            let metadata = get_metadata(Cursor::new(buffer.as_mut_slice()))?;

            // check that it matches expected values
            prop_assert_eq!(metadata.get_num_cameras(), 0);
            prop_assert_eq!(metadata.get_message_offsets().len(), 0);
        }

        #[test]
        fn skip_vision2010_msg(vision2010_msg in test_utils_message::random_vision2010_msg_strategy()) {
            let mut buffer = Vec::<u8>::new();
            let writer = Cursor::new(&mut buffer);

            let mut writer = LabelerDataWriter::new(writer)?;
            writer.add_msg(vision2010_msg)?;
            drop(writer);

            // check header
            check_header(Cursor::new(buffer.as_mut_slice()))?;

            // Read the metadata message
            let metadata = get_metadata(Cursor::new(buffer.as_mut_slice()))?;

            // check that it matches expected values
            prop_assert_eq!(metadata.get_num_cameras(), 0);
            prop_assert_eq!(metadata.get_message_offsets().len(), 0);
        }

        #[test]
        fn skip_unknown_msg(unknown_msg in test_utils_message::random_unknown_msg_strategy()) {
            let mut buffer = Vec::<u8>::new();
            let writer = Cursor::new(&mut buffer);

            let mut writer = LabelerDataWriter::new(writer)?;
            writer.add_msg(unknown_msg)?;
            drop(writer);

            // check header
            check_header(Cursor::new(buffer.as_mut_slice()))?;

            // Read the metadata message
            let metadata = get_metadata(Cursor::new(buffer.as_mut_slice()))?;

            // check that it matches expected values
            prop_assert_eq!(metadata.get_num_cameras(), 0);
            prop_assert_eq!(metadata.get_message_offsets().len(), 0);
        }

        #[test]
        fn vision_before_running_is_skipped(camera_msgs in test_utils_labeler::no_camera_repeats_strategy(1, 8)) {
            let mut buffer = Vec::<u8>::new();

            let mut writer = LabelerDataWriter::new(Cursor::new(&mut buffer))?;
            for camera_msg in camera_msgs {
                writer.add_msg(camera_msg)?;
            }
            drop(writer);

            // check header
            check_header(Cursor::new(buffer.as_mut_slice()))?;

            // Read the metadata message
            let metadata = get_metadata(Cursor::new(buffer.as_mut_slice()))?;

            // check that it matches expected values
            prop_assert_eq!(metadata.get_num_cameras(), 0);
            prop_assert_eq!(metadata.get_message_offsets().len(), 0);
        }

        #[test]
        fn vision_while_not_running_is_skipped(frame_msgs in test_utils_labeler::random_not_running_frame_strategy(1, 10)) {
            let mut buffer = Vec::<u8>::new();

            let mut writer = LabelerDataWriter::new(Cursor::new(&mut buffer))?;
            for frame_msg in frame_msgs {
                writer.add_msg(frame_msg)?;
            }
            drop(writer);

            // check header
            check_header(Cursor::new(buffer.as_mut_slice()))?;

            // Read the metadata message
            let metadata = get_metadata(Cursor::new(buffer.as_mut_slice()))?;

            // check that it matches expected values
            prop_assert_eq!(metadata.get_num_cameras(), 0);
            prop_assert_eq!(metadata.get_message_offsets().len(), 0);
        }

        #[test]
        fn dumps_remaining_data_on_drop(running_ref_msg in test_utils_labeler::running_ref_msg(),
                                        camera_msgs in test_utils_labeler::no_camera_repeats_strategy(1, 10)) {
            let mut buffer = Vec::<u8>::new();

            let mut writer = LabelerDataWriter::new(Cursor::new(&mut buffer))?;
            writer.add_msg(running_ref_msg.clone())?;
            for camera_msg in camera_msgs.iter().cloned() {
                writer.add_msg(camera_msg)?;
            }
            drop(writer);

            // check header
            check_header(Cursor::new(buffer.as_mut_slice()))?;

            // Read the metadata message
            let metadata = get_metadata(Cursor::new(buffer.as_mut_slice()))?;

            // check matches expected values
            prop_assert_eq!(metadata.get_num_cameras(), (camera_msgs.len() - 1) as u32);
            prop_assert_eq!(metadata.get_message_offsets().len(), 1);

            let frame_group = get_frame_at(Cursor::new(buffer.as_mut_slice()), metadata.get_message_offsets()[0])?;
            prop_assert_eq!(frame_group.get_frames().len(), 1 + camera_msgs.len());

            let ref_frame = &frame_group.get_frames()[0];
            prop_assert_eq!(ref_frame.get_timestamp(), running_ref_msg.timestamp as u64);
            prop_assert!(ref_frame.has_referee_frame());
            if let MessageType::Refbox2013(ref ref_msg) = running_ref_msg.msg_type {
                prop_assert_eq!(ref_frame.get_referee_frame(), ref_msg);
            }

            for (camera_msg, camera_frame) in camera_msgs.iter().cloned().zip(&frame_group.get_frames()[1..]) {
                prop_assert_eq!(camera_frame.get_timestamp(), camera_msg.timestamp as u64);
                prop_assert!(camera_frame.has_vision_frame());
                if let MessageType::Vision2014(ref vision_msg) = camera_msg.msg_type {
                    prop_assert_eq!(camera_frame.get_vision_frame(), vision_msg);
                }
            }
        }

        #[test]
        fn dumps_frame_on_cam_duplicate(running_ref_msg in test_utils_labeler::running_ref_msg(),
                                        frame1_camera_msgs in test_utils_labeler::no_camera_repeats_strategy(1, 10),
                                        frame2_camera_msgs in test_utils_labeler::no_camera_repeats_strategy(1, 10)) {
            let mut buffer = Vec::<u8>::new();

            let mut writer = LabelerDataWriter::new(Cursor::new(&mut buffer))?;
            writer.add_msg(running_ref_msg.clone())?;
            for camera_msg in frame1_camera_msgs.iter().cloned() {
                writer.add_msg(camera_msg)?;
            }
            for camera_msg in frame2_camera_msgs.iter().cloned() {
                writer.add_msg(camera_msg)?;
            }
            drop(writer);

            // check header
            check_header(Cursor::new(buffer.as_mut_slice()))?;

            // Read the metadata message
            let metadata = get_metadata(Cursor::new(buffer.as_mut_slice()))?;

            // check matches expected values
            let expected_num_cameras = (std::cmp::max(frame1_camera_msgs.len(), frame2_camera_msgs.len()) - 1) as u32;
            prop_assert_eq!(metadata.get_num_cameras(), expected_num_cameras);
            prop_assert_eq!(metadata.get_message_offsets().len(), 2);

            let frame_group = get_frame_at(Cursor::new(buffer.as_mut_slice()), metadata.get_message_offsets()[0])?;
            prop_assert_eq!(frame_group.get_frames().len(), 1 + frame1_camera_msgs.len());

            let ref_frame = &frame_group.get_frames()[0];
            prop_assert_eq!(ref_frame.get_timestamp(), running_ref_msg.timestamp as u64);
            prop_assert!(ref_frame.has_referee_frame());
            if let MessageType::Refbox2013(ref ref_msg) = running_ref_msg.msg_type {
                prop_assert_eq!(ref_frame.get_referee_frame(), ref_msg);
            }

            for (camera_msg, camera_frame) in frame1_camera_msgs.iter().cloned().zip(&frame_group.get_frames()[1..]) {
                prop_assert_eq!(camera_frame.get_timestamp(), camera_msg.timestamp as u64);
                prop_assert!(camera_frame.has_vision_frame());
                if let MessageType::Vision2014(ref vision_msg) = camera_msg.msg_type {
                    prop_assert_eq!(camera_frame.get_vision_frame(), vision_msg);
                }
            }

            let frame_group = get_frame_at(Cursor::new(buffer.as_mut_slice()), metadata.get_message_offsets()[1])?;
            prop_assert_eq!(frame_group.get_frames().len(), frame2_camera_msgs.len());
            for (camera_msg, camera_frame) in frame2_camera_msgs.iter().cloned().zip(&frame_group.get_frames()[..]) {
                prop_assert_eq!(camera_frame.get_timestamp(), camera_msg.timestamp as u64);
                prop_assert!(camera_frame.has_vision_frame());
                if let MessageType::Vision2014(ref vision_msg) = camera_msg.msg_type {
                    prop_assert_eq!(camera_frame.get_vision_frame(), vision_msg);
                }
            }

        }

        #[test]
        fn dumps_frame_when_switching_to_new_running(running_ref_msg in test_utils_labeler::running_ref_msg(),
                                                     camera_msgs in test_utils_labeler::no_camera_repeats_strategy(2, 10),
                                                     new_running_ref_msg in test_utils_labeler::running_ref_msg()) {
            let first_stage = match running_ref_msg.msg_type {
                MessageType::Refbox2013(ref ref_msg) => Some(ref_msg.get_stage()),
                _ => None,
            };
            let second_stage = match new_running_ref_msg.msg_type {
                MessageType::Refbox2013(ref ref_msg) => Some(ref_msg.get_stage()),
                _ => None,
            };
            prop_assume!(first_stage != second_stage);

            let mut buffer = Vec::<u8>::new();

            let mut writer = LabelerDataWriter::new(Cursor::new(&mut buffer))?;
            writer.add_msg(running_ref_msg.clone())?;
            let num_taken_for_first_frame = 1;
            for camera_msg in camera_msgs.iter().take(num_taken_for_first_frame).cloned() {
                writer.add_msg(camera_msg)?;
            }
            writer.add_msg(new_running_ref_msg.clone())?;
            for camera_msg in camera_msgs.iter().skip(num_taken_for_first_frame).cloned() {
                writer.add_msg(camera_msg)?;
            }
            drop(writer);

            // check header
            check_header(Cursor::new(buffer.as_mut_slice()))?;

            // Read the metadata message
            let metadata = get_metadata(Cursor::new(buffer.as_mut_slice()))?;

            // check matches expected values
            let expected_num_cameras = (camera_msgs.len() - 1) as u32;
            prop_assert_eq!(metadata.get_num_cameras(), expected_num_cameras);
            prop_assert_eq!(metadata.get_message_offsets().len(), 2);

            let frame_group = get_frame_at(Cursor::new(buffer.as_mut_slice()), metadata.get_message_offsets()[0])?;
            prop_assert_eq!(frame_group.get_frames().len(), 1 + num_taken_for_first_frame);

            let ref_frame = &frame_group.get_frames()[0];
            prop_assert_eq!(ref_frame.get_timestamp(), running_ref_msg.timestamp as u64);
            prop_assert!(ref_frame.has_referee_frame());
            if let MessageType::Refbox2013(ref ref_msg) = running_ref_msg.msg_type {
                prop_assert_eq!(ref_frame.get_referee_frame(), ref_msg);
            }

            for (camera_msg, camera_frame) in camera_msgs
                .iter()
                .take(num_taken_for_first_frame)
                .cloned()
                .zip(&frame_group.get_frames()[1..]) {
                prop_assert_eq!(camera_frame.get_timestamp(), camera_msg.timestamp as u64);
                prop_assert!(camera_frame.has_vision_frame());
                if let MessageType::Vision2014(ref vision_msg) = camera_msg.msg_type {
                    prop_assert_eq!(camera_frame.get_vision_frame(), vision_msg);
                }
                }

            let frame_group = get_frame_at(Cursor::new(buffer.as_mut_slice()), metadata.get_message_offsets()[1])?;
            prop_assert_eq!(frame_group.get_frames().len(), 1 + (camera_msgs.len() - num_taken_for_first_frame));

            let ref_frame = &frame_group.get_frames()[0];
            prop_assert_eq!(ref_frame.get_timestamp(), new_running_ref_msg.timestamp as u64);
            prop_assert!(ref_frame.has_referee_frame());
            if let MessageType::Refbox2013(ref ref_msg) = new_running_ref_msg.msg_type {
                prop_assert_eq!(ref_frame.get_referee_frame(), ref_msg);
            }

            for (camera_msg, camera_frame) in camera_msgs
                .iter()
                .skip(num_taken_for_first_frame)
                .cloned()
                .zip(&frame_group.get_frames()[1..]) {
                prop_assert_eq!(camera_frame.get_timestamp(), camera_msg.timestamp as u64);
                prop_assert!(camera_frame.has_vision_frame());
                if let MessageType::Vision2014(ref vision_msg) = camera_msg.msg_type {
                    prop_assert_eq!(camera_frame.get_vision_frame(), vision_msg);
                }
            }

        }

    }

}
