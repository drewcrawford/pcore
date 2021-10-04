

use objr::bindings::*;
use crate::release_pool::ReleasePool;
use std::ffi::c_void;
use std::ops::Deref;

///An owned string type
#[derive(Debug)]
pub struct PString(StrongCell<NSString>);

//conversions to/from platform version
impl PString {
    ///Converts into a platform-specific representation
    pub fn into_platform_string(self) -> StrongCell<NSString> {
        self.0
    }
    ///Converts from a platform-specific representation
    pub fn from_platform_string(platform: StrongCell<NSString>) -> Self {
        PString(platform)
    }
}

impl From<PString> for StrongCell<NSString> {
    fn from(f: PString) -> Self {
        f.into_platform_string()
    }
}
impl From<StrongCell<NSString>> for PString {
    fn from(platform: StrongCell<NSString>) -> Self {
        PString::from_platform_string(platform)
    }
}

//conversions to/from String.
//note: not implemented as From/Into becuase of release_pool arg.
impl PString {
    ///On macOS, this operation requires a copy.
    pub fn from_string(s: String, pool: &ReleasePool) -> Self {
        Self(NSString::with_str_copy(&s, pool))
    }
    ///On macOS, this operation requires a copy.
    pub fn into_string(self, pool: &ReleasePool) -> String {
        self.0.to_str(pool).to_string()
    }
}

///Borrowed platform string type
///
/// On macOS, pointers to this type are pointers to &NSString
pub struct Pstr(*const c_void);

impl Deref for PString {
    type Target = Pstr;

    fn deref(&self) -> &Self::Target {
        let as_nsstring: &NSString = &self.0;
        Pstr::from_platform_str(as_nsstring)
    }
}

//conversions to/from platform types
impl Pstr {
    pub fn from_platform_str(source: &NSString) -> &Self {
        unsafe{ std::mem::transmute(source) }
    }
    pub fn as_platform_str(&self) -> &NSString {
        unsafe { std::mem::transmute(self) }
    }
}

impl<'a> From<&'a NSString> for &'a Pstr {
    fn from(source: &'a NSString) -> Self {
        Pstr::from_platform_str(source)
    }
}
impl<'a> From<&'a Pstr> for &'a NSString {
    fn from(source: &'a Pstr) -> Self {
        Pstr::as_platform_str(source)
    }
}

//conversion to/from string types
impl Pstr {
    ///Converts from &Pstr to &str
    /// This may involve a copy
    pub fn to_string(&self, pool: &ReleasePool) -> String {
        let nsstring = self.as_platform_str();
        nsstring.to_str(pool).to_owned()
    }
}

///Declares a compile-time pstr
/// ```
/// use pcore::pstr;
/// let my_string = pstr!("My test string");
/// ```
#[macro_export]
macro_rules! pstr {
    ($str:literal) => {
        pcore::string::Pstr::from_platform_str(objr::bindings::objc_nsstring!($str))
    }
}