extern crate proptest;
use crate::persistence::message;
use crate::test_utils::protos as test_utils_protos;
use proptest::prelude::*;

prop_compose! {
    pub fn random_msg_bytes_strategy(max_size: i32)
        (num_bytes in 0..max_size)
        (bytes in proptest::collection::vec(proptest::num::u8::ANY, num_bytes as usize)) -> Vec<u8> {
            bytes
        }
}

prop_compose! {
    pub fn random_msg_bytes_eof_strategy(max_size: i32)
        (num_bytes in 0..(max_size - 1), max_size in Just(max_size))
        (bytes in prop::collection::vec(proptest::num::u8::ANY, num_bytes as usize),
         max_size in Just(max_size)) -> (Vec<u8>, i32) {
            (bytes, max_size)
        }
}

prop_compose! {
    pub fn random_blank_msg_strategy()(timestamp in 0..std::i64::MAX) -> message::Message {
        message::Message { timestamp, msg_type: message::MessageType::Blank }
    }
}

prop_compose! {
    pub fn random_vision2010_msg_strategy()
        (timestamp in 0..std::i64::MAX,
         msg_bytes in random_msg_bytes_strategy(100)
        ) -> message::Message {
        message:: Message { timestamp, msg_type: message::MessageType::Vision2010(msg_bytes) }
    }
}

prop_compose! {
    pub fn random_unknown_msg_strategy()
        (timestamp in 0..std::i64::MAX,
         msg_bytes in random_msg_bytes_strategy(100)
        ) -> message::Message {
        message::Message { timestamp, msg_type: message::MessageType::Unknown(msg_bytes) }
    }
}

prop_compose! {
    pub fn random_refbox2013_msg_strategy()
        (timestamp in 0..std::i64::MAX,
         refbox_msg in test_utils_protos::refbox2013_strategy()
        ) -> message::Message {
            message::Message { timestamp, msg_type: message::MessageType::Refbox2013(refbox_msg)}
        }
}

prop_compose! {
    pub fn random_vision2014_msg_strategy()
        (timestamp in 0..std::i64::MAX,
         vision_msg in test_utils_protos::vision2014_strategy()
        ) -> message::Message {
            message::Message { timestamp, msg_type: message::MessageType::Vision2014(vision_msg)}
        }
}
