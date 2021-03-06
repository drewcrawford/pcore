use std::fmt::{Display, Formatter};
use windows::Win32::Foundation::WIN32_ERROR;

#[derive(Debug)]
pub struct Error(WIN32_ERROR);

impl Error {
    pub fn from_win32(platform: WIN32_ERROR) -> Self {
        Error(platform)
    }
    pub fn into_win32(self) -> WIN32_ERROR {
        self.0
    }
    pub fn as_win32(&self) -> &WIN32_ERROR {
        &self.0
    }
    ///Calls GetLastError.
    ///
    /// Using this in pcore avoids a whole class of problems of the form "both you and some dependency
    /// import WIN32_ERROR, but they're different types"
    pub fn win32_last() -> Self {
        use windows::Win32::Foundation::GetLastError;
        Error(unsafe{GetLastError()})
    }
}
impl From<WIN32_ERROR> for Error {
    fn from(e: WIN32_ERROR) -> Self {
        Error::from_win32(e)
    }
}
impl From<Error> for WIN32_ERROR {
    fn from(e: Error) -> Self {
        e.into_win32()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}",self))
    }
}

impl std::error::Error for Error {}