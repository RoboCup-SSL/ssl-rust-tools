#[macro_use]
extern crate failure;

pub mod persistence;
pub mod protos;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
