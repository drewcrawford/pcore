use crate::release_pool::ReleasePool;
use winbindings::Windows::Win32::Foundation::PWSTR;
use std::ffi::c_void;
use std::ops::Deref;
use widestring::{U16CStr, U16CString};

///An owned string type
#[derive(Debug)]
pub struct PString(Box<[u16]>);

//conversions to/from platform version
impl PString {
    ///Converts into a platform-specific representation
    ///
    /// # Safety
    /// This type is immutable.  So it can only be passed to parameter of type `LPCWSTR` (constant).
    ///
    /// If you pass it to a fn expecting a mutable string, result is undefined.
    pub unsafe fn into_platform_string(self) -> PWSTR {
        PWSTR(self.0.as_ptr() as *const _ as *mut _)
    }
    ///Converts from a platform-specific representation
    ///
    /// On Windows, this performs a copy.
    ///
    /// # Safety
    /// We will dereference the argument and we assume it's a utf16 null-terminated string
    pub unsafe fn from_platform_string(platform: PWSTR) -> Self {
        let u16_str = U16CStr::from_ptr_str(platform.0);
        let owned_str = u16_str.to_ucstring();
        let bytes = owned_str.into_vec().into_boxed_slice();
        Self(bytes)
    }
}


//conversions to/from String.
//note: not implemented as From/Into becuase of release_pool arg.
impl PString {
    ///On Windows, this operation involves a copy and encode
    pub fn from_string(s: String, _pool: &ReleasePool) -> Self {
        let cstr = U16CString::from_str(s).unwrap().into_vec().into_boxed_slice();
        Self(cstr)
    }
    ///On windows, this operation involves a copy and encode
    pub fn into_string(self, _pool: &ReleasePool) -> String {
        let cstr = unsafe{ U16CStr::from_ptr_str(self.0.as_ptr())};
        cstr.to_string().unwrap()
    }
}

///Borrowed platform string type
///
/// On windows, pointers to this type are PWSTR
pub struct Pstr(*const c_void);

impl Deref for PString {
    type Target = Pstr;

    fn deref(&self) -> &Self::Target {
        //we aren't mutating the string
        let u16_ptr = self.0.as_ptr();
        let u16 = unsafe{ U16CStr::from_ptr_str(u16_ptr)};
        Pstr::from_platform_str(u16)
    }
}

//conversions to/from platform types
impl Pstr {
    pub fn from_platform_str(source: &U16CStr) -> &Self {
        unsafe{ std::mem::transmute(source.as_ptr()) }
    }
    ///# Safety
    /// This type is immutable.  So it can only be passed to parameter of type `LPCWSTR` (constant).
    ///
    /// If you pass it to a fn expecting a mutable string, result is undefined.
    pub unsafe fn as_platform_str(&self) -> PWSTR {
        PWSTR(std::mem::transmute(self))
    }
}

//conversion to/from string types
impl Pstr {
    //note that on macOS, we generally can't create borrowed types directly, so a move like &str -> &Pstr is not allowed.
    ///Note that on windows, we can't implement `as_str`, we need `to_string()`.
    ///Converts from &Pstr to &str
    pub fn to_string(&self, _pool: &ReleasePool) -> String {
        //We aren't mutating the string
        let platform = unsafe{ self.as_platform_str()};
        unsafe{ U16CStr::from_ptr_str(platform.0)}.to_string().unwrap()
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
        {
            let coerce: &'static pcore::string::Pstr = unsafe{ std::mem::transmute(wchar::wchz!($str))};
            coerce
        }

    }
}

// let my_string