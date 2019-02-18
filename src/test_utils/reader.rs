extern crate proptest;
use super::message as test_utils_message;
use crate::persistence::message;
use proptest::prelude::*;


prop_compose! {
    pub fn random_messages(min_num_messages: usize, max_num_messages: usize)
        (num_messages in min_num_messages..max_num_messages)
        (messages in prop::collection::vec(prop_oneof![
            test_utils_message::random_blank_msg_strategy(),
            test_utils_message::random_vision2010_msg_strategy(),
            test_utils_message::random_unknown_msg_strategy(),
            test_utils_message::random_refbox2013_msg_strategy(),
            test_utils_message::random_vision2014_msg_strategy(),
        ], num_messages)) -> Vec<message::Message> {
            messages
        }

}
