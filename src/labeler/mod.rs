const LABELER_DATA_HEADER: [u8; 16] = [
    b'S', b'S', b'L', b'_', b'L', b'A', b'b', b'E', b'L', b'E', b'R', b'_', b'D', b'A', b'T', b'A',
];
const LABELER_DATA_VERSION: u32 = 1u32;

pub mod writer;

