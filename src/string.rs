/**
Provides toll-free bridging to OS-specific strings

A brief discussion of string types:

* [std::string::String] and [std::str::str] are UTF-8 encoded Rust types
* [std::ffi::OsString] and [std::ffi::OsStr] are WTF-8 encoded Rust types, some 'relaxed' utf8 encoding appropriate for use in filesystems
* `NSString`, and specifically Rust projections like `StrongCell<NSString>` are the preferred string type on macOS.  This is an opaque
   encoding, in some cases UTF-8 and in other cases UTF-16.
* `HSTRING`, `PWSTR`, etc., are the preferred string type on Windows, UTF-16

What we want is:
1.  Conversions between types are *possible* (potentially slowly, e.g. re-encoding the string)
2.  But they can be *eliminated* (e.g., by getting your value into the right format to start with)

To solve this, `pcore` implements a variety of 'API' types:

* [IntoParameterString] is a trait for use on function parameters.  Conforming types provide conversions to the platform string type
* [ParameterString] erases an [IntoParameterString] into a concrete type.  This is appropriate for short-term use such as struct fields in a
  builder pattern.
* [OwnedString] copies the storage from an [IntoParameterString] and has `'static` lifetime.
* [pstr!] is a macro that gets strings into the correct format at compile-time to avoid runtime encoding.  The return type conforms to [IntoParameterString].

Platforms may have additional types as needed
 */
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::*;