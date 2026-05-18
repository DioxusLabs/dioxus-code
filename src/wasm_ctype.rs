//! C ctype symbols needed by Arborium grammar scanners in WASM builds.

#![allow(missing_docs)]

use std::ffi::c_int;

#[inline]
const fn is_lower(c: c_int) -> bool {
    c >= b'a' as c_int && c <= b'z' as c_int
}

#[inline]
const fn is_upper(c: c_int) -> bool {
    c >= b'A' as c_int && c <= b'Z' as c_int
}

#[inline]
const fn is_digit(c: c_int) -> bool {
    c >= b'0' as c_int && c <= b'9' as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn isalpha(c: c_int) -> c_int {
    (is_lower(c) || is_upper(c)) as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn isupper(c: c_int) -> c_int {
    is_upper(c) as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn isxdigit(c: c_int) -> c_int {
    (is_digit(c)
        || (c >= b'a' as c_int && c <= b'f' as c_int)
        || (c >= b'A' as c_int && c <= b'F' as c_int)) as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn isspace(c: c_int) -> c_int {
    matches!(
        c,
        x if x == b' ' as c_int
            || x == b'\t' as c_int
            || x == b'\n' as c_int
            || x == b'\r' as c_int
            || x == 0x0c
            || x == 0x0b
    ) as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn tolower(c: c_int) -> c_int {
    if is_upper(c) {
        c + (b'a' - b'A') as c_int
    } else {
        c
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn iswpunct(wc: u32) -> c_int {
    matches!(wc, 0x21..=0x2f | 0x3a..=0x40 | 0x5b..=0x60 | 0x7b..=0x7e) as c_int
}
