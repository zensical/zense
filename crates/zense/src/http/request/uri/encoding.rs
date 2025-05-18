// Copyright (c) 2024 Zensical <contributors@zensical.org>

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to
// deal in the Software without restriction, including without limitation the
// rights to use, copy, modify, merge, publish, distribute, sublicense, and/or
// sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NON-INFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS
// IN THE SOFTWARE.

// ----------------------------------------------------------------------------

//! Encoding.

use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet};
use std::borrow::Cow;

// ----------------------------------------------------------------------------
// Enums
// ----------------------------------------------------------------------------

/// Encoding kind.
#[derive(Clone, Copy, Debug)]
pub enum Kind {
    /// Request path.
    Path,
    /// Query string.
    Query,
}

// ----------------------------------------------------------------------------
// Constants
// ----------------------------------------------------------------------------

/// Characters that must be percent-encoded in paths.
const URI_PATH: &AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}')
    .add(b'~');

/// Characters that must be percent-encoded in query strings.
const URI_QUERY: &AsciiSet = &percent_encoding::CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'[')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

// ----------------------------------------------------------------------------
// Functions
// ----------------------------------------------------------------------------

/// Encodes a string.
///
/// The second argument specifies the kind of encoding to use, as this varies
/// by the usage context of the value, i.e., in paths or query strings.
#[inline]
#[must_use]
pub fn encode(value: &str, kind: Kind) -> Cow<str> {
    let set = match kind {
        Kind::Path => URI_PATH,
        Kind::Query => URI_QUERY,
    };

    // Encode using the specified set of characters
    utf8_percent_encode(value, set).into()
}

/// Decodes a string.
///
/// This function replaces invalid UTF-8 sequences with the Unicode replacement
/// character �, as otherwise, this would lead to a much less ergonomic API.
#[inline]
#[must_use]
pub fn decode(value: &str) -> Cow<str> {
    percent_decode_str(value).decode_utf8_lossy()
}
