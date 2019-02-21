use crate::persistence::message;
use crate::protos::log_labeler_data;
use crate::protos::messages_robocup_ssl_referee::{SSL_Referee_Command, SSL_Referee_Stage};
use crate::test_utils::message as test_utils_message;
use proptest::prelude::*;

prop_compose! {
    pub fn no_camera_repeats_strategy(min_num_cameras: u32, max_num_cameras: u32)
        (num_cameras in min_num_cameras..max_num_cameras)
        (mut vision_msgs in prop::collection::vec(test_utils_message::random_vision2014_msg_strategy(), num_cameras as usize)
        ) -> Vec<message::Message>{
            for (i, vision_msg) in vision_msgs.iter_mut().enumerate() {
                match vision_msg.msg_type {
                    message::MessageType::Vision2014(ref mut vision_msg) => vision_msg.mut_detection().set_camera_id(i as u32),
                    _ => panic!("Strategy generated non Vision2014 message type."),
                };
            }

            vision_msgs
        }
}

prop_compose! {
    pub fn not_running_stage_strategy()(ref_stage in prop_oneof![
             Just(SSL_Referee_Stage::NORMAL_FIRST_HALF_PRE),
             Just(SSL_Referee_Stage::NORMAL_HALF_TIME),
             Just(SSL_Referee_Stage::NORMAL_SECOND_HALF_PRE),
             Just(SSL_Referee_Stage::EXTRA_TIME_BREAK),
             Just(SSL_Referee_Stage::EXTRA_FIRST_HALF_PRE),
             Just(SSL_Referee_Stage::EXTRA_HALF_TIME),
             Just(SSL_Referee_Stage::EXTRA_SECOND_HALF_PRE),
             Just(SSL_Referee_Stage::PENALTY_SHOOTOUT_BREAK),
             Just(SSL_Referee_Stage::PENALTY_SHOOTOUT),
             Just(SSL_Referee_Stage::POST_GAME),
    ]) -> SSL_Referee_Stage {
        ref_stage
    }
}

prop_compose! {
    pub fn running_stage_strategy()(ref_stage in prop_oneof![
             Just(SSL_Referee_Stage::NORMAL_FIRST_HALF),
             Just(SSL_Referee_Stage::NORMAL_SECOND_HALF),
             Just(SSL_Referee_Stage::EXTRA_FIRST_HALF),
             Just(SSL_Referee_Stage::EXTRA_SECOND_HALF),
    ]) -> SSL_Referee_Stage {
        ref_stage
    }
}

prop_compose! {
    pub fn running_command_strategy()(ref_command in prop_oneof![
        Just(SSL_Referee_Command::NORMAL_START),
        Just(SSL_Referee_Command::FORCE_START),
        Just(SSL_Referee_Command::DIRECT_FREE_YELLOW),
        Just(SSL_Referee_Command::DIRECT_FREE_BLUE),
        Just(SSL_Referee_Command::INDIRECT_FREE_YELLOW),
        Just(SSL_Referee_Command::INDIRECT_FREE_BLUE),
    ]) -> SSL_Referee_Command {
        ref_command
    }
}

prop_compose! {
    pub fn not_running_command_strategy()(ref_command in prop_oneof![
        Just(SSL_Referee_Command::HALT),
        Just(SSL_Referee_Command::STOP),
        Just(SSL_Referee_Command::PREPARE_KICKOFF_YELLOW),
        Just(SSL_Referee_Command::PREPARE_KICKOFF_BLUE),
        Just(SSL_Referee_Command::PREPARE_PENALTY_YELLOW),
        Just(SSL_Referee_Command::PREPARE_PENALTY_BLUE),
        Just(SSL_Referee_Command::TIMEOUT_YELLOW),
        Just(SSL_Referee_Command::TIMEOUT_BLUE),
        Just(SSL_Referee_Command::GOAL_YELLOW),
        Just(SSL_Referee_Command::GOAL_BLUE),
        Just(SSL_Referee_Command::BALL_PLACEMENT_YELLOW),
        Just(SSL_Referee_Command::BALL_PLACEMENT_BLUE),
    ]) -> SSL_Referee_Command {
        ref_command
    }
}

prop_compose! {
    pub fn not_running_ref_stage_with_running_command_strategy()(
        ref_stage in not_running_stage_strategy(),
        ref_command in running_command_strategy()
    ) -> (SSL_Referee_Stage, SSL_Referee_Command) {
        (ref_stage, ref_command)
    }
}

prop_compose! {
    pub fn not_running_ref_stage_with_not_running_command_strategy()(
        ref_stage in not_running_stage_strategy(),
        ref_command in not_running_command_strategy()
    ) -> (SSL_Referee_Stage, SSL_Referee_Command) {
        (ref_stage, ref_command)
    }
}

prop_compose! {
    pub fn running_ref_stage_with_not_running_command_strategy()(
        ref_stage in running_stage_strategy(),
        ref_command in not_running_command_strategy()
    ) -> (SSL_Referee_Stage, SSL_Referee_Command) {
        (ref_stage, ref_command)
    }
}

prop_compose! {
    pub fn running_ref_msg()
        (mut ref_msg in test_utils_message::random_refbox2013_msg_strategy(),
         running_stage in running_stage_strategy(),
         running_command in running_command_strategy()) -> message::Message {
            match ref_msg.msg_type {
                message::MessageType::Refbox2013(ref mut ref_msg) => {
                    ref_msg.set_stage(running_stage);
                    ref_msg.set_command(running_command);
                },
                _ => panic!("Strategy generated non Refbox2013 message type."),
            };

            ref_msg
        }
}

prop_compose! {
    pub fn not_running_ref_msg()
        (mut ref_msg in test_utils_message::random_refbox2013_msg_strategy(),
         stage_and_command in prop_oneof![
             not_running_ref_stage_with_running_command_strategy(),
             not_running_ref_stage_with_not_running_command_strategy(),
             running_ref_stage_with_not_running_command_strategy(),
         ]) -> message::Message {
            match ref_msg.msg_type {
                message::MessageType::Refbox2013(ref mut ref_msg) => {
                    ref_msg.set_stage(stage_and_command.0);
                    ref_msg.set_command(stage_and_command.1);
                },
                _ => panic!("Strategy generated non Refbox2013 message type."),
            };

            ref_msg
        }
}

prop_compose! {
    pub fn random_running_frame_strategy(min_messages: u32, max_messages: u32)
        (num_messages in min_messages..max_messages)
        (mut frame_msgs in prop::collection::vec(prop_oneof![
            test_utils_message::random_vision2014_msg_strategy(),
            running_ref_msg(),
        ], num_messages as usize)
        ) -> Vec<message::Message> {
            let mut cam_id = 0u32;
            for frame_msg in frame_msgs.iter_mut() {
                match frame_msg.msg_type {
                    message::MessageType::Vision2014(ref mut vision_msg) => {
                        vision_msg.mut_detection().set_camera_id(cam_id);
                        cam_id += 1;
                    },
                    message::MessageType::Refbox2013(_) => {},
                    _ => panic!("Strategy generated non Vision2014 and non Refbox2013 message type."),
                };
            }

            frame_msgs
    }
}

prop_compose! {
    pub fn random_not_running_frame_strategy(min_messages: u32, max_messages: u32)
        (num_messages in min_messages..max_messages)
        (mut frame_msgs in prop::collection::vec(prop_oneof![
            test_utils_message::random_vision2014_msg_strategy(),
            not_running_ref_msg(),
        ], num_messages as usize)
        ) -> Vec<message::Message> {
            let mut cam_id = 0u32;
            for frame_msg in frame_msgs.iter_mut() {
                match frame_msg.msg_type {
                    message::MessageType::Vision2014(ref mut vision_msg) => {
                        vision_msg.mut_detection().set_camera_id(cam_id);
                        cam_id += 1;
                    },
                    message::MessageType::Refbox2013(_) => {},
                    _ => panic!("Strategy generated non Vision2014 and non Refbox2013 message type."),
                };
            }

            frame_msgs
    }
}

prop_compose! {
    pub fn random_no_ref_frame_group_msg_strategy(min_num_cameras: u32, max_num_cameras: u32)
        (camera_msgs in no_camera_repeats_strategy(min_num_cameras, max_num_cameras)
        ) -> log_labeler_data::LabelerFrameGroup {
            let mut frame_group_msg = log_labeler_data::LabelerFrameGroup::new();
            for camera_msg in camera_msgs {
                let mut frame_msg = log_labeler_data::LabelerFrame::new();
                frame_msg.set_timestamp(camera_msg.timestamp as u64);
                match camera_msg.msg_type {
                    message::MessageType::Vision2014(vision_msg) => frame_msg.set_vision_frame(vision_msg),
                    _ => panic!("Strategy returned non Vision2014 type."),
                };
                frame_group_msg.mut_frames().push(frame_msg)
            }

            frame_group_msg
        }
}

prop_compose! {
    pub fn random_frames_and_access_pattern_strategy(min_num_cameras: u32,
                                                     max_num_cameras: u32,
                                                     min_num_frames: usize,
                                                     max_num_frames: usize)
        (random_frame_group_msgs in
         proptest::collection::vec(random_no_ref_frame_group_msg_strategy(min_num_cameras, max_num_cameras),
                                   min_num_frames..max_num_frames))
        (access_pattern in proptest::collection::vec(0..random_frame_group_msgs.len(), random_frame_group_msgs.len()),
         random_frame_group_msgs in Just(random_frame_group_msgs)
        ) -> (Vec<log_labeler_data::LabelerFrameGroup>, Vec<usize>) {
             (random_frame_group_msgs, access_pattern)
        }
}

prop_compose! {
    pub fn random_frames_and_ranges_strategy(min_num_cameras: u32,
                                             max_num_cameras: u32,
                                             min_num_frames: usize,
                                             max_num_frames: usize)
        (random_frame_group_msgs in
         proptest::collection::vec(random_no_ref_frame_group_msg_strategy(min_num_cameras, max_num_cameras),
                                   min_num_frames..max_num_frames))
        (index1 in proptest::collection::vec(0..(random_frame_group_msgs.len() - 1), random_frame_group_msgs.len()),
         index2 in proptest::collection::vec(0..random_frame_group_msgs.len(), random_frame_group_msgs.len()),
         random_frame_group_msgs in Just(random_frame_group_msgs)
        ) -> (Vec<log_labeler_data::LabelerFrameGroup>, Vec<(usize, usize)>) {
            let range_indexes: Vec<(usize, usize)> = index1
                .iter()
                .cloned()
                .zip(index2)
                .filter(|(a, b)| *a != *b)
                .map(|(a, b)| (std::cmp::min(a, b),
                               std::cmp::max(a, b)))
                .collect();

            (random_frame_group_msgs, range_indexes)
        }
}
