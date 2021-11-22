- A safe idiomatic rust wrapper around https://github.com/sjkillen/libspnav-rust
- Does not expose X11 functions in libspnav
- Spacenav library for Rust
- Currently only difference from https://github.com/xanium4332/libspnav-rs is Connection struct with a finalizer to close the connection.
- That lib is older and more battle-tested; You should probably use it instead.
- Does not support the X11 functions
## Future:
- idiomatic async functions
- daemon-mode for rebinding mouse buttonsbuttons