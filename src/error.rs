/**
Defines [Error], a platform-specific error type.  The implementation
is toll-free bridged to some underlying platform concept of an error.
*/
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;