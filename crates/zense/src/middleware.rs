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

//! Middleware.

use std::fmt;

use crate::handler::{Handler, Result, Scope};
use crate::http::response::IntoResponse;
use crate::http::{Request, Response};

// ----------------------------------------------------------------------------
// Traits
// ----------------------------------------------------------------------------

/// Middleware.
///
/// Middlewares are the building blocks of any composable request processing
/// pipeline. They can be used to modify, handle or answer a given [`Request`],
/// with a [`Response`], or forward it to the next [`Handler`], which can be
/// another middleware or the final handler.
///
/// Note that a middleware consumes the request, which aligns with the idea of
/// a request moving through a pipeline. Besides closures which exactly match
/// the signature of [`Middleware::process`], this trait is implemented for
/// the following concepts:
///
/// - [`Stack`][]: Stack of middlewares.
/// - [`Router`][]: Router with parametrizable routes.
///
/// [`Router`]: crate::router::Router
/// [`Stack`]: crate::handler::Stack
pub trait Middleware: 'static {
    /// Processes the given request.
    ///
    /// This method is invoked with a request and is expected to either process
    /// the request and return a response, or pass it on to the given handler.
    /// Request processing is infallible, which means that errors must always
    /// be handled gracefully, e.g., by returning a 404 response.
    ///
    /// # Examples
    ///
    /// This example shows how to implement a teapot middleware responding with
    /// "418 I'm a Teapot" status code when the client tries to `GET /coffee`,
    /// while passing all other requests to the next [`Handler`]. Note that for
    /// routing, using [`Router`][] is usually a better choice.
    ///
    /// [`Router`]: crate::router::Router
    ///
    /// ```
    /// use zense::handler::{Handler, NotFound};
    /// use zense::middleware::Middleware;
    /// use zense::http::{Method, Request, Response, Status};
    ///
    /// // Define middleware
    /// struct Teapot;
    ///
    /// // Create middleware implementation
    /// impl Middleware for Teapot {
    ///     fn process(&self, req: Request, next: &dyn Handler) -> Response {
    ///         if req.method == Method::Get && req.uri.path == "/coffee" {
    ///             Response::new().status(Status::ImATeapot)
    ///         } else {
    ///             next.handle(req)
    ///         }
    ///     }
    /// }
    ///
    /// // Create request
    /// let req = Request::new()
    ///     .method(Method::Get)
    ///     .uri("/coffee");
    ///
    /// // Handle request with middleware
    /// let res = Teapot.process(req, &NotFound);
    /// assert_eq!(res.status, Status::ImATeapot);
    /// ```
    fn process(&self, req: Request, next: &dyn Handler) -> Response;
}

// ----------------------------------------------------------------------------

/// Attempt conversion into [`Middleware`].
pub trait TryIntoMiddleware: 'static {
    /// Output type of conversion.
    type Output: Middleware;

    /// Attempts to convert the implementor into a middleware.
    ///
    /// Since conversion can be fallible, it's a good idea to move validation
    /// prior to middleware instantiation into this method. This allows to keep
    /// the number of fallible methods as low as possible and allows for a more
    /// fluent API, as well as better error handling.
    ///
    /// Although middlewares are usually boxed, we return a concrete type, as
    /// it enables the compiler to employ monomorphization, if applicable.
    ///
    /// # Errors
    ///
    /// In case conversion fails, an error should be returned.
    fn try_into_middleware(self, scope: &Scope) -> Result<Self::Output>;
}

// ----------------------------------------------------------------------------
// Trait implementations
// ----------------------------------------------------------------------------

impl fmt::Debug for Box<dyn Middleware> {
    /// Formats the middleware for debugging.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Box<dyn Middleware>")
    }
}

// ----------------------------------------------------------------------------
// Blanket implementations
// ----------------------------------------------------------------------------

impl<F, R> Middleware for F
where
    F: Fn(Request, &dyn Handler) -> R + 'static,
    R: IntoResponse,
{
    #[inline]
    fn process(&self, req: Request, next: &dyn Handler) -> Response {
        self(req, next).into_response()
    }
}

// ----------------------------------------------------------------------------

impl<M> TryIntoMiddleware for M
where
    M: Middleware,
{
    type Output = Self;

    #[inline]
    fn try_into_middleware(self, _scope: &Scope) -> Result<Self> {
        Ok(self)
    }
}
