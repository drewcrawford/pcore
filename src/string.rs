/**
Provides [PString] and [Pstr], which are "platform string" types.  These types
store the string in whatever internal format is appropriate to the platform,
which may be different than the Rust internal format.

[PString] is analogous to [String], while [Pstr] is analogous to [str].  The [pstr!] macro allows declaring
compile-time strings.

Unlike [OSString], P types are represented in their native format.  Passing a P type
to an OS API should be zero cost.

Consequently, you may incur some cost (such as a copy) converting between P and Rust formats.
The exact nature of this cost varies per platform and per the specific conversion, and in some
cases can be eliminated.

P types do not express the full potential of the underlying format, but offer some reasonable subset
of functionality appropriate for cross-platform code.  A programmer willing to work with the platform
directly may be able to beat the performance of P types.

 */
mod macos;
pub use macos::*;