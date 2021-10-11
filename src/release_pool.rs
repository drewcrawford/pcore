/**
Provides a releasepool implementation.  On platforms other than macOS, all APIs are no-ops.

The benefit of this is to provide a common currency type, [ReleasePool], which cross-compiles everywhere
and enables releasepool-style optimizations in cross-platform code.

Generally, APIs with a likelihood of calling into a platform API should pass a [ReleasePool] parameter, this
enables releasepools to be re-used on macOS across implementations.
*/

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::*;