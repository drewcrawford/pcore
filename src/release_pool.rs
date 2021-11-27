/**
On macOS, an active release pool is generally required to call any OS API.  Marker types indicating that a release
pool is available are often required as an argument to OS-level bindings.

This module defines, [ReleasePool], a cross-platform type which compiels everywhere
and enables releasepool-style API design in cross-platform code.

On platforms which do not have release pools, these APIs and types are zero-cost abstractions
which have no effect.  However, using them enables designing APIs with macOS in mind.

Generally, APIs with a likelihood of calling into a platform API should take a [ReleasePool] parameter, this
enables releasepools to be re-used on macOS across heterogeneous implementations.
*/

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::*;