// Copyright 2018 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::error::Error as StdError;
use std::fmt;

use futures::{future, Future, IntoFuture};
use serde_json::Value as JsonValue;

use super::error::{Error, Never};

pub type Result = Box<Future<Item = Option<JsonValue>, Error = Error> + Send>;

pub trait Dispatch {
    type Error: Into<Box<StdError + Send + Sync>>;
    type Future: Future<Item = Option<JsonValue>, Error = Self::Error>;

    fn call(&mut self, req: JsonValue) -> Self::Future;
}

pub struct DispatchFn<F> {
    f: F,
}

pub fn dispatch_fn<F, D>(f: F) -> DispatchFn<F>
where
    F: Fn(JsonValue) -> D,
    D: IntoFuture, {
    DispatchFn {
        f,
    }
}

impl<F, Ret> Dispatch for DispatchFn<F>
where
    F: Fn(JsonValue) -> Ret,
    Ret: IntoFuture<Item = Option<JsonValue>>,
    Ret::Error: Into<Box<StdError + Send + Sync>>,
{
    type Error = Ret::Error;
    type Future = Ret::Future;

    fn call(&mut self, req: JsonValue) -> Self::Future {
        (self.f)(req).into_future()
    }
}

impl<F> IntoFuture for DispatchFn<F> {
    type Future = future::FutureResult<Self::Item, Self::Error>;
    type Item = Self;
    type Error = Never;

    fn into_future(self) -> Self::Future {
        future::ok(self)
    }
}

impl<F> fmt::Debug for DispatchFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("impl Dispatch").finish()
    }
}

/// An asynchronous constructor of `Dispatch`s.
pub trait NewDispatch {
    type Error: Into<Box<StdError + Send + Sync>>;
    type Dispatch: Dispatch<Error = Self::Error>;
    type Future: Future<Item = Self::Dispatch, Error = Self::InitError>;
    type InitError: Into<Box<StdError + Send + Sync>>;

    /// Create a new `Dispatch`.
    fn new_dispatch(&self) -> Self::Future;
}

impl<F, R, D> NewDispatch for F
where
    F: Fn() -> R,
    R: IntoFuture<Item = D>,
    R::Error: Into<Box<StdError + Send + Sync>>,
    D: Dispatch,
{
    type Error = D::Error;
    type Dispatch = D;
    type Future = R::Future;
    type InitError = R::Error;

    fn new_dispatch(&self) -> Self::Future {
        (*self)().into_future()
    }
}
