use objr::bindings::*;
use std::os::raw::c_ulong;
pub use objr::foundation::objc_nsstring as __objc_nsstring;

type NSUInteger = c_ulong;

///Type that can be converted into a platform string parameter.
///
/// The methods of this trait is platform-specific, so don't use them in cross-platform code.
/// The type itself however, is available everywhere.
///
/// Generally you want to accept a generic parameter of the form `<K: IntoParameterString>`.
pub trait IntoParameterString<'a> {
    ///Converts to an NSString.
    ///
    /// `storage`: Pass an instance of `MaybeUninit` here.
    fn into_nsstring(self, pool: &ActiveAutoreleasePool) -> StrongLifetimeCell<'a, NSString>;
    ///Erases to a parameter string
    fn into_parameter_string(self, pool: &ActiveAutoreleasePool) -> ParameterString<'a> where Self: Sized {
        ParameterString(self.into_nsstring(pool))
    }
}

//private extensions to NSString

objc_selector_group! {
    trait NSStringExtensionSelectors {
        @selector("initWithBytesNoCopy:length:encoding:freeWhenDone:")
    }
    impl NSStringExtensionSelectors for Sel {}
}

trait NSStringExtension {
    fn from_bytes_no_copy<'a>(bytes: &'a [u8], pool: &ActiveAutoreleasePool) -> StrongLifetimeCell<'a, NSString>;
}
impl NSStringExtension for NSString {
    fn from_bytes_no_copy<'a>(bytes: &'a [u8], pool: &ActiveAutoreleasePool) -> StrongLifetimeCell<'a, NSString> {
        unsafe {
            let uninit = Self::class().alloc(pool);
            let ptr = Self::perform(uninit, Sel::initWithBytesNoCopy_length_encoding_freeWhenDone(), pool, (bytes.as_ptr(), bytes.len() as NSUInteger, 4 as NSUInteger, false));
            NSString::assume_nonnil(ptr).assume_retained_limited()
        }
    }
}

impl<'a> IntoParameterString<'a> for &'a str {
    ///Borrow the bytes into an NSString instance.
    fn into_nsstring(self, pool: &ActiveAutoreleasePool) -> StrongLifetimeCell<'a, NSString> {
        NSString::from_bytes_no_copy(self.as_bytes(), pool)
    }
}

#[derive(PartialEq,Eq,Hash)]
pub struct ParameterString<'a>(StrongLifetimeCell<'a, NSString>);
impl<'a> IntoParameterString<'a> for ParameterString<'a> {
    fn into_nsstring(self, _pool: &ActiveAutoreleasePool) -> StrongLifetimeCell<'a, NSString> {
        self.0
    }
}
pub struct OwnedString(StrongCell<NSString>);
impl OwnedString {
    ///Create a new [OwnedString] by copying another string.
    pub fn new<'a, S: IntoParameterString<'a>>(string: S, pool: &ActiveAutoreleasePool) -> Self {
        let str = string.into_nsstring(pool);
        OwnedString(str.copy(pool))
    }
}
#[doc(hidden)]
pub struct StaticString(
    #[doc(hidden)]
    pub &'static NSString
);
impl IntoParameterString<'static> for StaticString {
    fn into_nsstring(self, _pool: &ActiveAutoreleasePool) -> StrongLifetimeCell<'static, NSString> {
        unsafe{StrongLifetimeCell::assume_retained_limited(self.0) }
    }
}
/// Provides a compile-time optimized path for parameter strings.
///
/// This macro is defined to return a type of [IntoPlatformString] that is reasonably fast
/// ```
/// use pcore::pstr;
/// let e = pstr!("test");
///
/// ```
#[macro_export]
macro_rules! pstr {
    ($expr:literal) => {
        pcore::string::StaticString(pcore::string::__objc_nsstring!($expr))
    }
}