use std::mem::MaybeUninit;
use std::hash::{Hash, Hasher};
use std::fmt::Formatter;
use std::ffi::c_void;
use windows::core::{HSTRING, Param};
use crate::release_pool::ReleasePool;
use windows::Win32::System::WinRT::{HSTRING_HEADER, WindowsCreateStringReference};


/**
For reasons we will never know, Microsoft decided to cripple string interop performance
for Rust specifically.

Suppose you have some large string in memory somewhere that you want to bridge into a windows HSTRING.
You can copy the thing at some time/cost, but why do that when you can borrow it?

As it happens, there's been a public Windows API for this since 2012.  It is [documented](https://docs.microsoft.com/en-us/windows/win32/api/winstring/nf-winstring-windowscreatestringreference),
[devblogged](https://devblogs.microsoft.com/oldnewthing/20160615-00/?p=93675), and [widely used](https://github.com/search?q=windowscreatestringreference&type=code)
in projects like [chromium](https://github.com/chromium/chromium/blob/72ceeed2ebcd505b8d8205ed7354e862b871995e/base/win/hstring_reference.cc) and [Qt](https://github.com/qt/qtbase/blob/9db7cc79a26ced4997277b5c206ca15949133240/src/plugins/platforms/windows/qwin10helpers.cpp).

However, calling this API using the official Rust bindings, crashes.  I [fixed the crash](https://github.com/microsoft/windows-rs/pull/1208), but MS wants to leave
the crash in as some kind of warning against faster strings performance.  The tagline is "any Windows API past, present, and future", but evidently not this one.

Since we can't play nice upstream, I have solved their crash here with some complexity.  I'm sorry you have to read it, and even sorrier
if you end up debugging it.  But I can promise if you send me PRs fixing my crashes I'll merge them :-)
*/
#[repr(C)]
pub struct ICantBelieveItsNotHString<'a>(&'a c_void,Option<Box<[u16]>>);
impl<'a> ICantBelieveItsNotHString<'a> {
    ///# Safety
    /// Can only pass a fast-pass hstring (e.g. created with `WindowsCreateStringReference`).
    /// Lifetime is not checked
    unsafe fn from_fastpass_hstring(hstring: HSTRING,backing_data:Option<Box<[u16]>>) -> Self {
        //read the inner field, this should be a pointer to the HSTRING header
        //HSTRING is defined #[repr(transparent)] so we should be able to transmute it to its field
        let field: *const c_void = std::mem::transmute(hstring);
        let r = ICantBelieveItsNotHString(&*field,backing_data);
        return r
    }
    fn as_hstring(&self) -> &HSTRING {
        /*
        So the basis of this trick is that we are layout-compatible with `::windows::HSTRING`.  It is
        `#[repr(transparent)]`, with one field (which internally points to an HSTRING_HEADER).

        We are #[repr(C)] with the same first field, which, under appropriate laws and regulations,
        means we can be cast into that type (though not the reverse).

        Thier layout is pretty fixed (I mean, they could change it, but it would be work)
        so it seems unlikely to me this will break.
         */
        unsafe {
            std::mem::transmute(self)
        }
    }
}
///This 'public', but `#[doc(hidden)]` API is required to define a type that can be passed
/// into windows-rs methods.
impl<'a> ::windows::core::IntoParam<'a, HSTRING> for &'a ICantBelieveItsNotHString<'a> {
    fn into_param(self) -> Param<'a, HSTRING> {
        /*This is really the whole secret, namely, that HSTRING crashes if it's dropped on a fast-pass
        string.  See https://github.com/microsoft/windows-rs/pull/1208 for "discussion"
        of their implementation.

        By passing `Param::Borrowed(&HSTRING)` here, we avoid actually creating an owned version of `::windows::HSTRING`,
        meaning that `.drop()` can never be called.

        Because the Drop trait can only be implemented on "structs, enums, or unions" there is no way for
        them to snoop on drop of &HSTRING.  There are some ways to break this, but I will not elaborate
        on them here.
        */
        Param::Borrowed(self.as_hstring())
    }
}



impl std::fmt::Debug for ICantBelieveItsNotHString<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:?}",self.as_hstring()))
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
# Implementation

On Windows, this type contains a slice of 0-terminated UTF-16, followed by owned storage (if needed, for example, for static strings).
To implement borrowed types, storage can be set to `None`.
 */
#[derive(Debug)]
pub struct ParameterString<'a>(&'a [u16],Option<Box<[u16]>>);
impl<'a> IntoParameterString<'a> for ParameterString<'a> {
    fn into_parameter_string(self, _pool: &ReleasePool) -> ParameterString<'a> {
        self
    }
}

//more or less, ParameterString gets its trait implementations from the `.0` field
impl<'a> PartialEq for ParameterString<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl<'a> Eq for ParameterString<'a> {}

impl<'a> Hash for ParameterString<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<'a> ParameterString<'a> {
    ///A view into the parameter string that is utf-16, null-terminated
    pub fn u16z_view(&self) -> U16ZKnownLength {
        U16ZKnownLength(self.0)
    }
}

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
///
pub trait IntoParameterString<'a> {
    ///Converts into an hstring 'trampoline'.  For reasons why this is not an hstring directly,
    /// see [ICantBelieveItsNotHString].
    /// * `header`: A pointer to `HSTRING_HEADER`.  In some cases, this will be used in the conversion.
    ///
    /// # Safety
    /// The resulting HSTRING will be valid only
    /// * For the lifetime of the header variable passed in.
    /// * For the lifetime of the `self` parameter.
    ///
    /// Note that in some cases, these constraints are not really required, however they are guaranteed and failure to uphold them
    /// may create arbitrary UB in the future.
    ///
    ///```
    /// use std::mem::MaybeUninit;
    /// use pcore::string::IntoParameterString;
    /// let e = "foo";
    /// let mut header = MaybeUninit::uninit();
    /// let h = unsafe{e.into_hstring_trampoline(&mut header)};
    /// ```
    unsafe fn into_hstring_trampoline<'h,'r: 'a + 'h>(self, header: &'h mut MaybeUninit<HSTRING_HEADER>) -> ICantBelieveItsNotHString<'r> where Self: Sized  + 'a {
        //not needed on windows
        let pool = ReleasePool::assuming_pool();
        let parameter_string = self.into_parameter_string(pool);
        let mut hstring = MaybeUninit::uninit();
        //ok to transmute here because windows won't mutate our string
        let pwstr = PWSTR(std::mem::transmute(parameter_string.0.as_ptr()));
        WindowsCreateStringReference(pwstr, parameter_string.0.len() as u32 - 1, header.assume_init_mut(), hstring.assume_init_mut()).unwrap();
        ICantBelieveItsNotHString::from_fastpass_hstring(hstring.assume_init(),parameter_string.1)
    }
    ///Converts into a null-terminated pwstr.
    ///
    /// # Safety
    /// * The resulting pwstr will be valid only
    /// * For the lifetime of the `self` parameter
    /// * When the underlying PWSTR is not modified.  e.g., you must pass it to a function of type LPCWSTR
    ///
    /// Note that the type returned here may be different than the PWSTR in use in some other library.  Therefore,
    /// you may need to transmute "this" type into "that" type.
    unsafe fn into_unsafe_const_pwzstr(self) -> PWSTR where Self: Sized {
        //not needed on windows
        let pool = ReleasePool::assuming_pool();
        let parameter_string = self.into_parameter_string(pool);
        PWSTR(std::mem::transmute(parameter_string.0.as_ptr()))
    }

    ///Converts into an erased type
    ///
    /// For compatibility with macOS, this takes a releasepool parameter
    fn into_parameter_string(self, pool: &ReleasePool) -> ParameterString<'a>;
}

///Implements conversions, primarily by copying
impl<'a> IntoParameterString<'a> for &'a str {
    fn into_parameter_string(self, _pool: &ReleasePool) -> ParameterString<'a> {
        //convert to utf16z
        let encode = self.encode_utf16();
        //reserve capacity for size_hint + 1 for null
        let size_hint = encode.size_hint();
        let mut v = Vec::with_capacity(size_hint.1.unwrap_or(size_hint.0) + 1);
        for item in encode {
            v.push(item);
        }
        v.push(0);
        let boxed_slice = v.into_boxed_slice();
        //fool rust into letting us take &temp
        let slice_ptr = boxed_slice.as_ptr();
        let slice_len = boxed_slice.len();
        ParameterString(unsafe{std::slice::from_raw_parts(slice_ptr, slice_len)}, Some(boxed_slice))
    }
}
///An instance created by the [pstr!] macro.  This is a static string.
///
/// Instances can be created with the [pstr!] macro.
#[derive(Copy,Clone,Debug)]
pub struct PStr(pub &'static [u16]);
impl IntoParameterString<'static> for PStr {
    fn into_parameter_string(self,_pool: &ReleasePool) -> ParameterString<'static> {
        ParameterString(self.0, None)
    }
}

impl ToString for PStr {
    fn to_string(&self) -> String {
        unsafe{widestring::U16CStr::from_slice_with_nul_unchecked(self.0)}.to_string().unwrap()
    }
}

impl<'a> IntoParameterString<'a> for &'a std::path::Path {
    fn into_parameter_string(self, _pool: &ReleasePool) -> ParameterString<'a> {
        let encoded = widestring::U16CString::from_os_str(self.as_os_str()).unwrap();
        let boxed = encoded.into_vec_with_nul().into_boxed_slice();
        //fool rust into letting us take &temp
        let slice_ptr = boxed.as_ptr();
        let slice_len = boxed.len();
        ParameterString(unsafe{std::slice::from_raw_parts(slice_ptr, slice_len)}, Some(boxed))
    }
}

///Represents a null-terminated string of length known at runtime (but not compile-time)
pub struct U16ZKnownLength<'a>(&'a [u16]);
impl<'a> U16ZKnownLength<'a> {
    ///Converts to an owned type, by cloning.  This erases the lifetime of the type
    pub fn to_owned(&self) -> OwnedString {
        OwnedString(self.0.clone().to_vec().into_boxed_slice())
    }
    ///Returns a utf16 null-terminated slice
    pub fn utf16z_slice(&self) -> &[u16] {
        self.0
    }
}
impl<'a> IntoParameterString<'a> for U16ZKnownLength<'a> {
    fn into_parameter_string(self,_pool: &ReleasePool) -> ParameterString<'a> {
        ParameterString(self.0, None)
    }
}

///Represents a null-terminated string of length not known at runtime
pub struct U16ZErasedLength<'a>(&'a [u16]);
impl<'a> U16ZErasedLength<'a> {
    ///# Safety
    /// * The slice must have a null-terminator
    pub unsafe fn with_u16_z_unknown_length(slice: &'a [u16]) -> Self {
        Self(slice)
    }
    fn len_with_z(&self) -> usize {
        //scan the string for \0
        for i in 0..self.0.len() {
            if self.0[i] == 0 {
                return i+1;
            }
        }
        panic!("U16Z is not null-terminated")
    }
    pub fn find_length(&self) -> U16ZKnownLength {
        let actual_len = self.len_with_z();
        let adjusted_slice = unsafe{std::slice::from_raw_parts(self.0.as_ptr(), actual_len)};
        U16ZKnownLength(&adjusted_slice)
    }
}
impl<'a> std::fmt::Debug for U16ZErasedLength<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let actual_len = self.len_with_z() - 1;
        let adjusted_slice = unsafe{std::slice::from_raw_parts(self.0.as_ptr(), actual_len)};
        let s = String::from_utf16(adjusted_slice).unwrap();
        f.write_str(&s)
    }
}
impl<'a> IntoParameterString<'a> for &U16ZErasedLength<'a> {
    fn into_parameter_string(self,_pool: &ReleasePool) -> ParameterString<'a> {
        let actual_len = self.len_with_z();
        let actual_slice = unsafe{std::slice::from_raw_parts(self.0.as_ptr(), actual_len)};
        ParameterString(actual_slice, None)
    }
    unsafe fn into_unsafe_const_pwzstr(self) -> PWSTR where Self: Sized {
        //faster path without finding out length
        PWSTR(std::mem::transmute(self.0.as_ptr()))
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
pub struct OwnedString(Box<[u16]>);
impl OwnedString {
    pub fn new<'a, S: IntoParameterString<'a>>(string: S, pool: &ReleasePool) -> Self {
        let parameter_string = string.into_parameter_string(pool);
        let boxed = match parameter_string {
            ParameterString(_, Some(b)) => {
                //move the box into the new type
                b
            }
            ParameterString(slice,None) => {
                //will require a clone
                slice.to_owned().into_boxed_slice()
            }
        };
        Self(boxed)
    }
}
impl ToString for OwnedString {
    fn to_string(&self) -> String {
        let s = &self.0.split_last().unwrap().1;
        String::from_utf16(s).unwrap()
    }
}

impl<'a> IntoParameterString<'a> for &'a OwnedString {
    fn into_parameter_string(self,_pool: &ReleasePool) -> ParameterString<'a> {
        ParameterString(&self.0, None)
    }
}
impl std::fmt::Debug for OwnedString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = &self.0.split_last().unwrap().1;
        let str = String::from_utf16(s).unwrap();
        f.write_str(&str)
    }
}

#[doc(hidden)]
pub use wchar::wchz as __wchz;
use windows::Win32::Foundation::PWSTR;


/// Provides a compile-time optimized path for parameter strings.
///
/// This macro is defined to return a [PStr]
/// ```
/// use pcore::pstr;
/// let e = pstr!("test");
///
/// ```
#[macro_export]
macro_rules! pstr {
    ($expr:literal) => {
        {
            pcore::string::PStr(pcore::string::__wchz!($expr))
        }

    }
}

#[test] fn str_into() {
    use windows::Foundation::Uri;
    let f = "https://sealedabstract.com";
    let mut h = MaybeUninit::uninit();
    let hstr = unsafe{f.into_hstring_trampoline(&mut h)};
    println!("hstr {:?}",hstr);
    //call some API that requires IntoParam
    Uri::CreateUri(&hstr).unwrap();
}

#[test] fn static_into() {
    use windows::Foundation::Uri;
    let f = pstr!("https://sealedabstract.com");
    let mut h = MaybeUninit::uninit();
    let hstr = unsafe{f.into_hstring_trampoline(&mut h)};
    println!("hstr {:?}",hstr);
    //call some API that requires IntoParam
    Uri::CreateUri(&hstr).unwrap();
}

#[test] fn path() {
    use std::path::PathBuf;
    use std::str::FromStr;
    let p = PathBuf::from_str("test").unwrap();
    let path = p.as_path();
    let release_pool = unsafe{ReleasePool::new()};
    let parameter_string = path.into_parameter_string(&release_pool);
    let mut header = MaybeUninit::uninit();
    let _ = unsafe{parameter_string.into_hstring_trampoline(&mut header)};
}

#[test] fn to_string() {
    let p = pstr!("Hello world");
    assert_eq!(p.to_string(), "Hello world");
}