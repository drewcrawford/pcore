use objr::bindings::*;
use std::os::raw::c_ulong;
pub use objr::foundation::objc_nsstring as __objc_nsstring;

type NSUInteger = c_ulong;

///Type that can be converted into a platform string parameter.
///
/// The methods of this trait is platform-specific, so don't use them in cross-platform code.
/// The type itself however, is available everywhere.
///
/// Generally you want to accept a generic parameter of the form `<S: IntoParameterString>`, for example
///
///```
/// use pcore::string::IntoParameterString;
/// fn foo<'a, S: IntoParameterString<'a>>(s: S) {
///    //use `s`
/// }
/// ```
///
/// This trait is implemented by various standard library types ([str], [String], etc.) but also the output of [pstr!],
/// platform-specific string types, and various others.  Any of these conforming types may be passed to the function directly.
/// Encoding or conversion will be performed automatically if required.
///
/// For best performance, prefer passing a value of:
/// 1.  [PStr], if the string can be known at compile-time
/// 2.  [IntoParameterString], if one is available
/// 3.  A platform-specific type, such as the result of calling an OS API.
/// 4.  A type with the native encoding, such as UTF16 (on Windows), etc.
/// 5.  A standard library type, like [String].
pub trait IntoParameterString<'a> {
    ///Converts to an NSString.
    ///
    /// `storage`: Pass an instance of `MaybeUninit` here.
    fn into_nsstring(self, pool: &ActiveAutoreleasePool) -> StrongLifetimeCell<'a, NSString>;
    ///Erases to a parameter string
    ///
    /// Design note.  You may try to get away with not passing a release pool in here.  In practice, objc APIs assume you did,
    /// and to whatever extent you don't have a releasepool active is not documented.
    ///
    /// I looked into it, and a lot of things you might call in here (such as `[[NSString alloc] init]` don't actually try to autorelease anything.
    /// However I'm not confident that every implementation of this trait ever, and all the OS changes year to year, won't brick me in some way.
    /// So it's safer to just pass this in even in cases where it's not strictly necessary, vs either allocating a new one, or dealing
    /// with the consequences of not having an active pool.
    ///
    /// This clutters up APIs a bit, but it's safer and not that much more work, and I don't think perf benefits are really there
    /// to cut corners on it
    fn into_parameter_string(self, pool: &ActiveAutoreleasePool) -> ParameterString<'a> where Self: Sized {
        ParameterString(self.into_nsstring(pool))
    }
}

//private extensions to NSString

objc_selector_group! {
    trait NSStringExtensionSelectors {
        @selector("initWithBytesNoCopy:length:encoding:freeWhenDone:")
        @selector("initWithBytesNoCopy:length:encoding:deallocator:")
    }
    impl NSStringExtensionSelectors for Sel {}
}

trait NSStringExtension {
    fn from_bytes_no_copy<'a>(bytes: &'a [u8], pool: &ActiveAutoreleasePool) -> StrongLifetimeCell<'a, NSString>;
    fn from_bytes_no_copy_deallocator<'a>(bytes: &'a [u8], deallocator: &Deallocator, pool: &ActiveAutoreleasePool) -> StrongLifetimeCell<'a, NSString>;
}

blocksr::once_escaping!(Deallocator(ptr: *const core::ffi::c_void, len: NSUInteger) -> ());
unsafe impl Arguable for &Deallocator{}

impl NSStringExtension for NSString {
    fn from_bytes_no_copy<'a>(bytes: &'a [u8], pool: &ActiveAutoreleasePool) -> StrongLifetimeCell<'a, NSString> {
        unsafe {
            let uninit = Self::class().alloc(pool);
            let ptr = Self::perform(uninit, Sel::initWithBytesNoCopy_length_encoding_freeWhenDone(), pool, (bytes.as_ptr().assume_nonmut_perform(), bytes.len() as NSUInteger, 4 as NSUInteger, false));
            NSString::assume_nonnil(ptr).assume_retained_limited()
        }
    }
    fn from_bytes_no_copy_deallocator<'a>(bytes: &'a [u8], deallocator: &Deallocator, pool: &ActiveAutoreleasePool) -> StrongLifetimeCell<'a, NSString> {
        unsafe {
            let uninit = Self::class().alloc(pool);
            let ptr = Self::perform(uninit, Sel::initWithBytesNoCopy_length_encoding_deallocator(), pool, (bytes.as_ptr().assume_nonmut_perform(), bytes.len() as NSUInteger, 4 as NSUInteger, deallocator));
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
impl IntoParameterString<'static> for String {
    fn into_nsstring(self, pool: &ActiveAutoreleasePool) -> StrongLifetimeCell<'static, NSString> {
        //I think this is pinned for the lifetime of the string
        let bytes = unsafe{std::slice::from_raw_parts(self.as_ptr(), self.len())};
        let block = unsafe{Deallocator::new(|_,_| {
            std::mem::drop(self);
        })};
        NSString::from_bytes_no_copy_deallocator(bytes, &block, pool)
    }
}

/**
A type that erases [IntoParameterString] into a concrete type with a named lifetime.

This type is appropriate for use in a builder pattern, or other cases where the string
will be stored for a short time.

# Example
```
use pcore::string::{ParameterString,IntoParameterString};
use pcore::release_pool::ReleasePool;
struct StringBuilder<'a> {
     inner: ParameterString<'a>,
}
impl<'a> StringBuilder<'a> {
    fn new<S: IntoParameterString<'a>>(string: S, pool: &ReleasePool) -> Self {
        Self { inner: string.into_parameter_string(pool) }
    }
}
```
*/
#[derive(PartialEq,Eq,Hash)]
pub struct ParameterString<'a>(StrongLifetimeCell<'a, NSString>);
impl<'a> IntoParameterString<'a> for ParameterString<'a> {
    fn into_nsstring(self, _pool: &ActiveAutoreleasePool) -> StrongLifetimeCell<'a, NSString> {
        self.0
    }
}
/**
An owned string type.  This may be appropriate for long-term string storage in a struct field.

In some cases the implementation may copy the string into the type, in other cases there may
be some platform-specific trick that can avoid a copy in certain cases.

# Example
```
use pcore::string::{OwnedString,IntoParameterString};
use pcore::release_pool::ReleasePool;
struct MyType {
     inner: OwnedString,
}
impl MyType {
    fn new<'a, S: IntoParameterString<'a>>(string: S, pool: &ReleasePool) -> Self {
        Self { inner: OwnedString::new(string,pool) }
    }
}
```
*/
pub struct OwnedString(StrongCell<NSString>);
impl OwnedString {
    ///Create a new [OwnedString] by copying another string.
    pub fn new<'a, S: IntoParameterString<'a>>(string: S, pool: &ActiveAutoreleasePool) -> Self {
        let str = string.into_nsstring(pool);
        OwnedString(str.copy(pool))
    }
}
///An instance created by the [pstr!] macro.  This is a static string.
///
/// Instances can be created with the [pstr!] macro.
#[derive(Copy,Clone,Debug)]
pub struct PStr(
    #[doc(hidden)]
    pub &'static NSString
);
impl IntoParameterString<'static> for PStr {
    fn into_nsstring(self, _pool: &ActiveAutoreleasePool) -> StrongLifetimeCell<'static, NSString> {
        unsafe{StrongLifetimeCell::assume_retained_limited(self.0) }
    }
}

//need to re-export this so it's usable from our macro...
#[doc(hidden)]
pub use objr as __objr;
/// Provides a compile-time string.  The result of this expression conforms to [IntoParameterString].
///
/// This macro is defined to return type `Pstr`.  Generally, this is the supported constructor of that type.
/// ```
/// use pcore::pstr;
/// let e = pstr!("test");
///
/// ```
#[macro_export]
macro_rules! pstr {
    ($expr:literal) => {
        {
            use pcore::string::__objr as objr;
            pcore::string::PStr(pcore::string::__objc_nsstring!($expr))
        }
    }
}

#[test] fn from_owned_string() {
    let f = "my test string".to_owned();
    fn thunk<'a, I: IntoParameterString<'a>>(i: I) {
        autoreleasepool(|pool| {
            assert!(i.into_nsstring(pool).to_string() == "my test string")
        })
    }
    thunk(f);
}