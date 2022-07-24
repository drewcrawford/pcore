/**
# pcore

pcore is a **p**latform **core** library.  It provides string and error, and similar fundamental types as suitable for
cross-platform software.

Unlike standard library types, pcore types are zero-cost wrappers around the platform's native type.  For example,
Windows APIs take UTF-16 strings, while macOS APIs take NSString.  pcore types compile down to the appropriate
encoding on every platform, enabling cross-platform code that has native performance.

pcore is free for noncommercial and "small commercial" use.

## Strings

See module [string]

On Windows, pcore uses UTF-16 encoding.  On macOS, pcore uses [objr](https://github.com/drewcrawford/objr) as a
zero-cost bridge to NSString.  Linux and other platforms are planned.

Notably, `pstr!("hello world")` will statically allocate an appropriate string *at compile-time*,
which is perfect for string constants and similar use cases.

The Windows behavior relies on a few quirks of the [windows-rs crate](https://github.com/microsoft/windows-rs) to bridge UTF16 in directly.
In particular, upstream [does not support bridging UTF-16 strings](https://github.com/microsoft/windows-rs/pull/1208), even though Microsoft [recommends](https://docs.microsoft.com/en-us/windows/win32/intl/unicode) that
applications on their platform "should use UTF-16 as their internal data representation".  As such,
this crate is the only way I'm aware of to get zero-cost performance using first-party bindings.

pcore introduces a family of string types for various uses including as API parameters, builders, and more.
For more information, see the documentation for the `string` module.

## Release pools

See module [release_pool].

On macOS, an active release pool is generally required to call any OS API.  Marker types indicating that a release
pool is available are often required as an argument to OS-level bindings.

pcore implements a cross-platform API to acquire a marker type.  On macOS, this wraps the [objr](https://github.com/drewcrawford/objr)
release pool implementation.  On other platforms, this is API has no effect and is a zero-cost abstraction.

Allowing release pool semantics to be expressed on platforms that don't support them helps when writing cross-platform
code in which one implementation will need to use them.

## Errors

See module [error].

On macOS, `Error` wraps `NSError`.  On Windows, currently the error type wraps WIN32_ERROR.  It is unclear at this moment
the right design for non-Win32 error types, but I will come up with one.

*/
pub mod string;
pub mod release_pool;
pub mod error;
extern crate self as pcore;
extern crate core;

