/**
pcore is a tiny library abstracting over certain *core* differences in *platforms*.  pcore includes:

* [release_pool], an API for release pools, a common feature of macOS APIs.
* [string], a set of platform string formats, including compile-time strings
* [error], a platform-specific error type
*/
pub mod string;
pub mod release_pool;
pub mod error;
extern crate self as pcore;

