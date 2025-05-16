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

//! HTTP response.

use crate::http::{Header, Status};

use super::Response;

// ----------------------------------------------------------------------------
// Traits
// ----------------------------------------------------------------------------

/// Extension trait for the [`Response`] type.
pub trait ResponseExt: Sized {
    /// Creates a response from a status code.
    ///
    /// This is a convenience method to create a response with a status code
    /// and a text body, particularly useful for error handling.
    #[must_use]
    fn from_status(status: Status) -> Response {
        let content = status.name();
        Response::new()
            .status(status)
            .header(Header::ContentType, "text/plain; charset=utf-8")
            .header(Header::ContentLength, content.len())
            .body(content)
    }
}

// ----------------------------------------------------------------------------
// Blanket implementations
// ----------------------------------------------------------------------------

impl ResponseExt for Response {}
