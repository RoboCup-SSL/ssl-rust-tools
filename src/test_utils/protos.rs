extern crate proptest;
use crate::protos::messages_robocup_ssl_detection;
use crate::protos::messages_robocup_ssl_geometry;
use crate::protos::messages_robocup_ssl_referee;
use crate::protos::messages_robocup_ssl_wrapper;
use proptest::prelude::*;


fn one_of_protobuf_enum<T: protobuf::ProtobufEnum>() -> proptest::strategy::Union<Just<T>>
where
    T: std::fmt::Debug,
{
    proptest::strategy::Union::new(T::values().iter().map(|x| Just(*x)))
}

prop_compose! {
    pub fn refbox2013_team_info_strategy()(name in "\\PC*",
                                       score in proptest::num::u32::ANY,
                                       red_cards in proptest::num::u32::ANY,
                                       yellow_cards in proptest::num::u32::ANY,
                                       timeouts in proptest::num::u32::ANY,
                                       timeout_time in proptest::num::u32::ANY,
                                       goalie in proptest::num::u32::ANY
    ) -> messages_robocup_ssl_referee::SSL_Referee_TeamInfo {
        let mut team_info_msg = messages_robocup_ssl_referee::SSL_Referee_TeamInfo::new();
        team_info_msg.set_name(name);
        team_info_msg.set_score(score);
        team_info_msg.set_red_cards(red_cards);
        team_info_msg.set_yellow_cards(yellow_cards);
        team_info_msg.set_timeouts(timeouts);
        team_info_msg.set_timeout_time(timeout_time);
        team_info_msg.set_goalie(goalie);

        team_info_msg
    }
}

prop_compose! {
    pub fn refbox2013_strategy()(packet_timestamp in proptest::num::u64::ANY,
                             stage in one_of_protobuf_enum::<messages_robocup_ssl_referee::SSL_Referee_Stage>(),
                             command in one_of_protobuf_enum::<messages_robocup_ssl_referee::SSL_Referee_Command>(),
                             command_counter in proptest::num::u32::ANY,
                             command_timestamp in proptest::num::u64::ANY,
                             yellow in refbox2013_team_info_strategy(),
                             blue in refbox2013_team_info_strategy()
    ) -> messages_robocup_ssl_referee::SSL_Referee {
        let mut ref_msg = messages_robocup_ssl_referee::SSL_Referee::new();
        ref_msg.set_packet_timestamp(packet_timestamp);
        ref_msg.set_stage(stage);
        ref_msg.set_command(command);
        ref_msg.set_command_counter(command_counter);
        ref_msg.set_command_timestamp(command_timestamp);
        ref_msg.set_yellow(yellow);
        ref_msg.set_blue(blue);

        ref_msg
    }
}

prop_compose! {
    pub fn detection_frame_strategy()(frame_number in proptest::num::u32::ANY,
                                  t_capture in proptest::num::f64::NORMAL | proptest::num::f64::POSITIVE | proptest::num::f64::NEGATIVE,
                                  t_sent in proptest::num::f64::NORMAL | proptest::num::f64::POSITIVE | proptest::num::f64::NEGATIVE,
                                  camera_id in proptest::num::u32::ANY
    ) -> messages_robocup_ssl_detection::SSL_DetectionFrame {
        let mut detection_msg = messages_robocup_ssl_detection::SSL_DetectionFrame::new();
        detection_msg.set_frame_number(frame_number);
        detection_msg.set_t_capture(t_capture);
        detection_msg.set_t_sent(t_sent);
        detection_msg.set_camera_id(camera_id);

        detection_msg
    }
}

prop_compose! {
    pub fn geometry_field_size_strategy()(field_length in proptest::num::i32::ANY,
                                      field_width in proptest::num::i32::ANY,
                                      goal_width in proptest::num::i32::ANY,
                                      goal_depth in proptest::num::i32::ANY,
                                      boundary_width in proptest::num::i32::ANY
    ) -> messages_robocup_ssl_geometry::SSL_GeometryFieldSize {
        let mut field_size_msg = messages_robocup_ssl_geometry::SSL_GeometryFieldSize::new();
        field_size_msg.set_field_length(field_length);
        field_size_msg.set_field_width(field_width);
        field_size_msg.set_goal_width(goal_width);
        field_size_msg.set_goal_depth(goal_depth);
        field_size_msg.set_boundary_width(boundary_width);

        field_size_msg
    }
}

prop_compose! {
    pub fn geometry_data_strategy()(field in geometry_field_size_strategy()
    ) -> messages_robocup_ssl_geometry::SSL_GeometryData {
        let mut geometry_msg = messages_robocup_ssl_geometry::SSL_GeometryData::new();
        geometry_msg.set_field(field);

        geometry_msg
    }
}

prop_compose! {
    pub fn vision2014_strategy()(detection in detection_frame_strategy(),
                             geometry in geometry_data_strategy()
    ) -> messages_robocup_ssl_wrapper::SSL_WrapperPacket {
        let mut vision_msg = messages_robocup_ssl_wrapper::SSL_WrapperPacket::new();
        vision_msg.set_detection(detection);
        vision_msg.set_geometry(geometry);

        vision_msg
    }
}
