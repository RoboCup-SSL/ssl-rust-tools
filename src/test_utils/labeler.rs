use crate::persistence::message;
use crate::protos::messages_robocup_ssl_referee::SSL_Referee_Stage;
use crate::test_utils::message as test_utils_message;
use crate::test_utils::protos as test_utils_protos;
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
    pub fn running_ref_msg()
        (mut ref_msg in test_utils_message::random_refbox2013_msg_strategy(),
         running_stage in prop_oneof![
             Just(SSL_Referee_Stage::NORMAL_FIRST_HALF),
             Just(SSL_Referee_Stage::NORMAL_SECOND_HALF),
             Just(SSL_Referee_Stage::EXTRA_FIRST_HALF),
             Just(SSL_Referee_Stage::EXTRA_SECOND_HALF),
         ]) -> message::Message {
            match ref_msg.msg_type {
                message::MessageType::Refbox2013(ref mut ref_msg) => ref_msg.set_stage(running_stage),
                _ => panic!("Strategy generated non Refbox2013 message type."),
            };

            ref_msg
        }
}

prop_compose! {
    pub fn not_running_ref_msg()
        (mut ref_msg in test_utils_message::random_refbox2013_msg_strategy(),
         not_running_stage in prop_oneof![
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
         ]) -> message::Message {
            match ref_msg.msg_type {
                message::MessageType::Refbox2013(ref mut ref_msg) => ref_msg.set_stage(not_running_stage),
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
