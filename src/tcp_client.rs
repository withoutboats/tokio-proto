use std::{fmt, io};
use std::sync::Arc;
use std::net::SocketAddr;
use std::marker::PhantomData;

use BindClient;
use tokio_core::reactor::Handle;
use tokio_core::net::{TcpStream, TcpStreamNew};
use tokio_service::NewService;
use futures::{Future, Poll, Async};

// TODO: add configuration, e.g.:
// - connection timeout
// - multiple addresses
// - request timeout

// TODO: consider global event loop handle, so that providing one in the builder
// is optional

/// Builds client connections to external services.
///
/// To connect to a service, you need a *client protocol* implementation; see
/// the crate documentation for guidance.
///
/// At the moment, this builder offers minimal configuration, but more will be
/// added over time.
#[derive(Debug)]
pub struct TcpClient<Kind, P> {
    _kind: PhantomData<Kind>,
    proto: Arc<P>,
}

/// A TcpClient bound to an address and event loop.
///
/// This implements `NewService`, and can be used as a factory for new client
/// services.
#[derive(Debug)]
pub struct BoundTcpClient<Kind, P> {
    _kind: PhantomData<Kind>,
    proto: Arc<P>,
    addr: SocketAddr,
    handle: Handle,
}

/// A future for establishing a client connection.
///
/// Yields a service for interacting with the server.
pub struct Connect<Kind, P> {
    _kind: PhantomData<Kind>,
    proto: Arc<P>,
    socket: TcpStreamNew,
    handle: Handle,
}

impl<Kind, P> Future for Connect<Kind, P> where P: BindClient<Kind, TcpStream> {
    type Item = P::BindClient;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<P::BindClient, io::Error> {
        let socket = try_ready!(self.socket.poll());
        Ok(Async::Ready(self.proto.bind_client(&self.handle, socket)))
    }
}

impl<Kind, P> TcpClient<Kind, P> where P: BindClient<Kind, TcpStream> {
    /// Create a builder for the given client protocol.
    ///
    /// To connect to a service, you need a *client protocol* implementation;
    /// see the crate documentation for guidance.
    pub fn new(protocol: P) -> TcpClient<Kind, P> {
        TcpClient {
            _kind: PhantomData,
            proto: Arc::new(protocol)
        }
    }

    /// Establish a connection to the given address.
    ///
    /// # Return value
    ///
    /// Returns a future for the establishment of the connection. When the
    /// future completes, it yields an instance of `Service` for interacting
    /// with the server.
    pub fn connect(&self, addr: &SocketAddr, handle: &Handle) -> Connect<Kind, P> {
        Connect {
            _kind: PhantomData,
            proto: self.proto.clone(),
            socket: TcpStream::connect(addr, handle),
            handle: handle.clone(),
        }
    }

    /// Bind this client to an address and handle.
    ///
    /// # Return value
    ///
    /// Returns a factory for constructing new client services, which
    /// implements the `NewService` trait.
    pub fn bind(&self, addr: SocketAddr, handle: Handle) -> BoundTcpClient<Kind, P> {
        BoundTcpClient {
            _kind: PhantomData,
            proto: self.proto.clone(),
            addr: addr,
            handle: handle,
        }
    }
}

impl<Kind, P> NewService for BoundTcpClient<Kind, P> where P: BindClient<Kind, TcpStream> {
    type Request = P::ServiceRequest;
    type Response = P::ServiceResponse;
    type Error = P::ServiceError;
    type Instance = P::BindClient;
    type Future = Connect<Kind, P>;

    fn new_service(&self) -> Self::Future {
        Connect {
            _kind: PhantomData,
            proto: self.proto.clone(),
            socket: TcpStream::connect(&self.addr, &self.handle),
            handle: self.handle.clone(),
        }
    }
}

impl<Kind, P> fmt::Debug for Connect<Kind, P> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Connect {{ ... }}")
    }
}
