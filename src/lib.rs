#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "gui")]
pub mod gui;
pub mod labeler;
pub mod persistence;
pub mod player;
pub mod protos;

#[cfg(test)]
pub mod test_utils;
