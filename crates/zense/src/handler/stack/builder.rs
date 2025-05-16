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

//! Stack builder.

use std::str::FromStr;

use crate::handler::matcher::{Matcher, Route};
use crate::handler::{Error, Result, Scope, TryIntoHandler};
use crate::middleware::{Middleware, TryIntoMiddleware};

use super::factory::Factory;
use super::Stack;

// ----------------------------------------------------------------------------
// Structs
// ----------------------------------------------------------------------------

/// Stack builder.
#[derive(Debug)]
pub struct Builder {
    /// Middleware factories.
    middlewares: Vec<Box<dyn Factory>>,
}

// ----------------------------------------------------------------------------
// Implementations
// ----------------------------------------------------------------------------

impl Builder {
    /// Creates a stack builder.
    pub(crate) fn new() -> Self {
        Self { middlewares: Vec::new() }
    }

    /// Adds a middleware to the stack.
    ///
    /// Note that [`Builder::with`] is the canonical way to compose stacks from
    /// middlewares. This method is solely used internally by the [`Router`][].
    ///
    /// [`Router`]: crate::router::Router
    pub(crate) fn push<M>(&mut self, middleware: M)
    where
        M: TryIntoMiddleware,
    {
        self.middlewares.push(Box::new(|scope: &Scope| {
            middleware
                .try_into_middleware(scope)
                .map(|middleware| Box::new(middleware) as Box<dyn Middleware>)
        }));
    }

    /// Adds a middleware to the stack.
    ///
    /// Anything that can be converted into a [`Middleware`] can be added to
    /// the stack, including middlewares, routers, stacks and closures.
    ///
    /// # Errors
    ///
    /// Errors returned by [`TryIntoMiddleware`] are passed through.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use zense::handler::{Handler, Stack};
    /// use zense::http::{Method, Request, Response, Status};
    ///
    /// // Create stack with middleware
    /// let stack = Stack::new()
    ///     .with(|req: Request, next: &dyn Handler| {
    ///         if req.method == Method::Get && req.uri.path == "/coffee" {
    ///             Response::new().status(Status::ImATeapot)
    ///         } else {
    ///             next.handle(req)
    ///         }
    ///     });
    /// # Ok(())
    /// # }
    /// ```
    #[inline]
    pub fn with<M>(mut self, middleware: M) -> Self
    where
        M: TryIntoMiddleware,
    {
        self.push(middleware);
        self
    }
}

// ----------------------------------------------------------------------------
// Trait implementations
// ----------------------------------------------------------------------------

impl TryIntoMiddleware for Builder {
    type Output = Stack;

    /// Attempts to convert the stack into a middleware.
    ///
    /// # Errors
    ///
    /// In case conversion fails, an [`Error`][] is returned.
    ///
    /// [`Error`]: crate::handler::Error
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use zense::handler::{Handler, Scope, Stack};
    /// use zense::http::{Method, Request, Response, Status};
    /// use zense::middleware::TryIntoMiddleware;
    ///
    /// // Create scope
    /// let scope = Scope::default();
    ///
    /// // Create stack with middleware
    /// let stack = Stack::new()
    ///     .with(|req: Request, next: &dyn Handler| {
    ///         if req.method == Method::Get && req.uri.path == "/coffee" {
    ///             Response::new().status(Status::ImATeapot)
    ///         } else {
    ///             next.handle(req)
    ///         }
    ///     })
    ///     .try_into_middleware(&scope)?;
    /// # Ok(())
    /// # }
    /// ```
    fn try_into_middleware(self, scope: &Scope) -> Result<Self::Output> {
        let route = scope.route.as_ref();

        // If the stack is part of a router, we create a matcher that checks if
        // the router's base path matches the request path as a prefix
        let matcher = route
            .map(|base| -> Result<_> {
                let mut matcher = Matcher::new();
                let rest = Route::from_str("/{*rest}")
                    .map_err(|err| Error::Matcher(err.into()))?;

                // Middlewares do not receive path parameters, which is why we
                // just use a wildcard to implement prefix matching on paths
                matcher
                    .add(base.append(rest), ())
                    .map_err(Into::into)
                    .map(|()| matcher)
            })
            .transpose()?;

        // Create and collect middlewares into a stack
        let iter = self.middlewares.into_iter().map(|f| f(scope));
        iter.collect::<Result<_>>()
            .map(|middlewares| Stack { middlewares, matcher })
    }
}

impl TryIntoHandler for Builder {
    type Output = Stack;

    /// Attempts to convert the stack into a handler.
    ///
    /// This method is equivalent to calling [`Stack::try_into_middleware`]
    /// with [`Scope::default`], scoping all middlewares to `/`.
    ///
    /// # Errors
    ///
    /// In case conversion fails, an [`Error`][] is returned.
    ///
    /// [`Error`]: crate::handler::Error
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// use zense::handler::{Handler, Stack, TryIntoHandler};
    /// use zense::http::{Method, Request, Response, Status};
    ///
    /// // Create stack with middleware
    /// let stack = Stack::new()
    ///     .with(|req: Request, next: &dyn Handler| {
    ///         if req.method == Method::Get && req.uri.path == "/coffee" {
    ///             Response::new().status(Status::ImATeapot)
    ///         } else {
    ///             next.handle(req)
    ///         }
    ///     })
    ///     .try_into_handler()?;
    /// # Ok(())
    /// # }
    /// ```
    fn try_into_handler(self) -> Result<Self::Output> {
        let scope = Scope::default();
        self.try_into_middleware(&scope)
    }
}
