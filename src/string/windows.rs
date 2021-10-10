use std::mem::MaybeUninit;
use winbindings::Windows::Win32::System::WinRT::{HSTRING_HEADER,WindowsCreateStringReference};
use winbindings::HSTRING;
use winbindings::Windows::Win32::Foundation::PWSTR;
use std::hash::{Hash, Hasher};
use std::fmt::Formatter;


///Erased `ToParameterString`, this is an appropriate type for storing in a Builder or other short-term storage.
///
/// The type may refer to external storage; in that case the lifetime parameter specifies
/// the storage lifetime.
pub struct ParameterString<'a>(&'a [u16],Option<Box<[u16]>>);
impl<'a> IntoParameterString<'a> for ParameterString<'a> {
    fn into_parameter_string(self) -> ParameterString<'a> {
        Self(self.0, self.1.clone())
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
/// Generally you want to accept a generic parameter of the form `<K: IntoParameterString>`.
pub trait IntoParameterString<'a> {
    ///Converts into the hstring
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
    /// use pcore::string::ToParameterString;
    /// let e = "foo";
    /// let mut header = MaybeUninit::uninit();
    /// let h = unsafe{e.to_unsafe_hstring(&mut header)};
    /// ```
    ///
    /// # Note
    /// On Windows, each bindings crate is likely to declare its own HSTRING type.  Therefore,
    /// you may need to transmute the return value into "your" type, as distinct from the HSTRING
    /// type from pcore.
    unsafe fn into_unsafe_hstring(self, header: &mut MaybeUninit<HSTRING_HEADER>) -> HSTRING where Self: Sized {
        let parameter_string = self.into_parameter_string();
        let mut hstring = MaybeUninit::uninit();
        //ok to transmute here because windows won't mutate our string
        let pwstr = PWSTR(std::mem::transmute(parameter_string.0.as_ptr()));
        WindowsCreateStringReference(pwstr, parameter_string.0.len() as u32 - 1, header.assume_init_mut(), hstring.assume_init_mut()).unwrap();
        hstring.assume_init()
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
        let parameter_string = self.into_parameter_string();
        PWSTR(std::mem::transmute(parameter_string.0.as_ptr()))
    }

    ///Converts into an erased type
    fn into_parameter_string(self) -> ParameterString<'a>;
}

///Implements conversions, primarily by copying
impl<'a> IntoParameterString<'static> for &str {
    fn into_parameter_string(self) -> ParameterString<'static> {
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
#[doc(hidden)]
pub struct StaticStr(pub &'static [u16]);
impl IntoParameterString<'static> for StaticStr {
    fn into_parameter_string(self) -> ParameterString<'static> {
        ParameterString(self.0, None)
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
    fn into_parameter_string(self) -> ParameterString<'a> {
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
    fn into_parameter_string(self) -> ParameterString<'a> {
        let actual_len = self.len_with_z();
        let actual_slice = unsafe{std::slice::from_raw_parts(self.0.as_ptr(), actual_len)};
        ParameterString(actual_slice, None)
    }
    unsafe fn into_unsafe_const_pwzstr(self) -> PWSTR where Self: Sized {
        //faster path without finding out length
        PWSTR(std::mem::transmute(self.0.as_ptr()))
    }
}

///Owned type, 'static lifetime, suitable for long-term storage
///
/// On windows, implemented as a null-terminated utf16 string
pub struct OwnedString(Box<[u16]>);

impl<'a> IntoParameterString<'a> for &'a OwnedString {
    fn into_parameter_string(self) -> ParameterString<'a> {
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



/// Provides a compile-time optimized path for parameter strings.
///
/// This macro is defined to return a type of `ToParameterString` that is reasonably fast
/// ```
/// use pcore::pstr;
/// let e = pstr!("test");
///
/// ```
#[macro_export]
macro_rules! pstr {
    ($expr:literal) => {
        {
            let static_arr = wchar::wchz!($expr);
            pcore::string::StaticStr(wchar::wchz!($expr))
        }

    }
}

#[test] fn str_into() {
    let f = "test";
    let mut h = MaybeUninit::uninit();
    let hstr = unsafe{f.into_unsafe_hstring(&mut h)};
    println!("hstr {:?}",hstr);
}
