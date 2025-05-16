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

//! HTTP request.

use httparse::Status;
use std::borrow::Cow;
use std::fmt;
use std::str::{self, FromStr};

use super::component::{Header, Method};

mod error;
mod headers;
mod uri;

pub use error::{Error, Result};
pub use headers::Headers;
pub use uri::{Query, Uri};

// ----------------------------------------------------------------------------
// Structs
// ----------------------------------------------------------------------------

/// HTTP request.
///
/// The regular way to create a [`Request`] is to use [`Request::from_bytes`],
/// which parses a given slice of bytes. The returned [`Request`] is bound to
/// the lifetime of the byte slice, avoiding unnecessary allocations where
/// possible, except for the [`BTreeMap`][] used for headers.
///
/// [`BTreeMap`]: std::collections::BTreeMap
///
/// # Examples
///
/// ```
/// use zense::http::{Method, Request};
///
/// // Create request
/// let res = Request::new()
///     .method(Method::Get)
///     .uri("/");
/// ```
#[derive(Clone, Debug)]
pub struct Request<'a> {
    /// Request method.
    pub method: Method,
    /// Request URI.
    pub uri: Uri<'a>,
    /// Request headers.
    pub headers: Headers<'a>,
    /// Request body.
    pub body: Cow<'a, [u8]>,
}

// ----------------------------------------------------------------------------
// Implementations
// ----------------------------------------------------------------------------

impl<'a> Request<'a> {
    /// Creates a request.
    ///
    /// # Examples
    ///
    /// ```
    /// use zense::http::Request;
    ///
    /// // Create request
    /// let req = Request::new();
    /// ```
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a request from the given bytes.
    ///
    /// HTTP requests are parsed using the [`httparse`] crate, which is one of
    /// the few dependencies that we rely on as it provides an efficient, fast,
    /// and well-tested parser. The returned [`Request`] will be bound to the
    /// lifetime of the input, avoiding allocations where possible.
    ///
    /// After parsing, basic validation is performed on the request path to
    /// ensure it doesn't exceed 4kb in size and doesn't attempt traversal.
    ///
    /// # Errors
    ///
    /// This method returns [`Error::Incomplete`], if the given buffer contained
    /// insufficient data to provide a meaningful answer, [`Error::Parser`], if
    /// the buffer contained invalid data, and [`Error::Component`], when the
    /// parsed request contains an invalid [`Method`] or [`Header`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use zense::http::{Method, Request};
    ///
    /// // Create request from bytes
    /// let req = Request::from_bytes(b"GET / HTTP/1.1\r\n\r\n")?;
    /// assert_eq!(req.method, Method::Get);
    /// assert_eq!(req.uri.path, "/");
    /// # Ok(())
    /// # }
    /// ```
    #[allow(clippy::missing_panics_doc)]
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self> {
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);

        // Parse request using the `httparse` crate, and create a new request
        // from the parsed data. Note that we only use the `httparse` crate and
        // not the `http` crate, as the later provides a rather inconvenient
        // interface for writing middlewares comfortably.
        match req.parse(bytes).map_err(Error::from)? {
            Status::Partial => Err(Error::Incomplete),
            Status::Complete(n) => {
                let body = Cow::Borrowed(&bytes[n..]);

                // Unpack request method and URI - if parsing succeeded, we can
                // be confident that method and path, both options, must exist
                let method = req.method.expect("invariant").parse()?;
                let uri = Uri::from(req.path.expect("invariant"));

                // Unpack request headers - ignore header parsing errors and
                // unknown headers, as it doesn't matter for request handling
                let iter = req.headers.iter();
                let headers = iter
                    .take_while(|header| !header.name.is_empty())
                    .filter_map(|header| {
                        str::from_utf8(header.value).ok().and_then(|value| {
                            Header::from_str(header.name)
                                .map(|name| (name, value))
                                .ok()
                        })
                    })
                    .collect();

                // Ensure request path doesn't exceed 4kb - most web servers
                // allow up to 4-8kb, so 4kb should be more than enough for us
                if uri.path.len() > 4 * 1024 {
                    return Err(Error::Security("exceeds size of 4kb"));
                }

                // Ensure request path doesn't attempt traversal - a quick and
                // dirty check, and yes, there might be false positives
                if uri.path.contains("..") {
                    return Err(Error::Security("path traversal"));
                }

                // Return request
                Ok(Request { method, uri, headers, body })
            }
        }
    }
}

impl<'a> Request<'a> {
    /// Sets the method of the request.
    ///
    /// # Examples
    ///
    /// ```
    /// use zense::http::{Method, Request};
    ///
    /// // Create request and set method
    /// let res = Request::new()
    ///     .method(Method::Post);
    /// ```
    #[inline]
    #[must_use]
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// Sets the URI of the request.
    ///
    /// # Examples
    ///
    /// ```
    /// use zense::http::{Method, Request};
    ///
    /// // Create request and set URI
    /// let res = Request::new()
    ///     .uri("/");
    /// ```
    #[inline]
    #[must_use]
    pub fn uri<U>(mut self, uri: U) -> Self
    where
        U: Into<Uri<'a>>,
    {
        self.uri = uri.into();
        self
    }

    /// Adds a header to the request.
    ///
    /// # Examples
    ///
    /// ```
    /// use zense::http::{Header, Request};
    ///
    /// // Create request and add header
    /// let req = Request::new()
    ///     .header(Header::Accept, "text/plain");
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    #[inline]
    #[must_use]
    pub fn header<V>(mut self, header: Header, value: V) -> Self
    where
        V: ToString,
    {
        self.headers.put(header, value.to_string());
        self
    }

    /// Sets the body of the request.
    ///
    /// # Examples
    ///
    /// ```
    /// use zense::http::Request;
    ///
    /// // Create request and set body
    /// let res = Request::new()
    ///     .body("Hello world");
    /// ```
    #[inline]
    #[must_use]
    pub fn body<B>(mut self, body: B) -> Self
    where
        B: Into<Vec<u8>>,
    {
        self.body = Cow::Owned(body.into());
        self
    }
}

// ----------------------------------------------------------------------------
// Trait implementations
// ----------------------------------------------------------------------------

impl Default for Request<'_> {
    /// Creates a default request.
    ///
    /// # Examples
    ///
    /// ```
    /// use zense::http::Request;
    ///
    /// // Create request
    /// let req = Request::default();
    /// ```
    #[inline]
    fn default() -> Self {
        Self {
            method: Method::Get,
            uri: Uri::default(),
            headers: Headers::default(),
            body: Cow::Borrowed(&[]),
        }
    }
}

// ----------------------------------------------------------------------------

impl fmt::Display for Request<'_> {
    /// Formats the response for display.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} HTTP/1.1\r\n", self.method, self.uri)?;
        write!(f, "{}\r\n", self.headers)?;
        write!(f, "[Body: {} bytes]\r\n", self.body.len())
    }
}
