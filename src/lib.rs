#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;

pub mod persistence;
pub mod player;
pub mod protos;

#[cfg(test)]
pub mod test_utils;
