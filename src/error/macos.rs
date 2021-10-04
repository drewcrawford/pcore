use foundationr::*;
use objr::bindings::StrongCell;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct Error(StrongCell<NSError>);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}",self))
    }
}

impl std::error::Error for Error {}