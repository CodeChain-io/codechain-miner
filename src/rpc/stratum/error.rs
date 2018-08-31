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
use std::io;

type Cause = Box<StdError + Send + Sync>;

/// Represents errors that can occur handling Stratum streams.
pub struct Error {
    inner: Box<ErrorImpl>,
}

struct ErrorImpl {
    kind: Kind,
    cause: Option<Cause>,
}

#[derive(Debug, PartialEq)]
pub enum Kind {
    /// A message reached EOF, but is not complete.
    Incomplete,
    /// Indicates a connection is closed.
    Closed,
    /// An `io::Error` that occurred while trying to read or write to a network stream.
    Io,
    /// Error occurred while connecting.
    Connect,
    /// Error occurred while authenticating.
    Authenticate,
    /// Error trying to call `Executor::execute`.
    Execute,
}

impl Error {
    fn new(kind: Kind, cause: Option<Cause>) -> Error {
        Error {
            inner: Box::new(ErrorImpl {
                kind,
                cause,
            }),
        }
    }

    pub fn new_io(cause: io::Error) -> Error {
        Error::new(Kind::Io, Some(cause.into()))
    }

    pub fn new_connect<E: Into<Cause>>(cause: E) -> Error {
        Error::new(Kind::Connect, Some(cause.into()))
    }

    pub fn new_closed() -> Error {
        Error::new(Kind::Closed, None)
    }

    pub fn new_incomplete() -> Error {
        Error::new(Kind::Incomplete, None)
    }

    pub fn new_authenticate() -> Error {
        Error::new(Kind::Authenticate, None)
    }

    pub fn new_execute() -> Error {
        Error::new(Kind::Execute, None)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Error").field("kind", &self.inner.kind).field("cause", &self.inner.cause).finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref cause) = self.inner.cause {
            write!(f, "{}: {}", self.description(), cause)
        } else {
            f.write_str(self.description())
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self.inner.kind {
            Kind::Incomplete => "parsed JSON message from remote is incomplete",
            Kind::Closed => "connection closed",
            Kind::Connect => "an error occurred trying to connect",
            Kind::Authenticate => "an error occurred trying to authenticate",
            Kind::Execute => "executor failed to spawn task",
            Kind::Io => "an IO error occurred",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        self.inner.cause.as_ref().map(|cause| &**cause as &StdError)
    }
}

#[derive(Debug)]
pub enum Never {}

impl fmt::Display for Never {
    fn fmt(&self, _: &mut fmt::Formatter) -> fmt::Result {
        match *self {}
    }
}

impl StdError for Never {
    fn description(&self) -> &str {
        match *self {}
    }
}
