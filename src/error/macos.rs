use objr::bindings::StrongCell;
use std::fmt::{Display, Formatter};
use objr::foundation::NSError;

#[derive(Debug)]
pub struct Error(StrongCell<NSError>);

impl Error {
    pub fn from_nserror(platform: StrongCell<NSError>) -> Self {
        Error(platform)
    }
    pub fn into_nserror(self) -> StrongCell<NSError> {
        self.0
    }
}
impl From<StrongCell<NSError>> for Error {
    fn from(e: StrongCell<NSError>) -> Self {
        Error::from_nserror(e)
    }
}
impl From<Error> for StrongCell<NSError> {
    fn from(e: Error) -> Self {
        e.into_nserror()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}",self))
    }
}

impl std::error::Error for Error {}