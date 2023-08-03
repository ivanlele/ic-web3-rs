//! `Web3` implementation

mod accounts;
mod eth;

pub use eth::Eth;
pub use accounts::Accounts;

use crate::{
    error,
    transports::ic_http_client::CallOptions,
    types::{Bytes, TransactionReceipt, TransactionRequest, U64},
    Error, RequestId, Transport,
};
use futures::Future;
use jsonrpc_core::types::Call;
use std::time::Duration;

/// Common API for all namespaces
pub trait Namespace<T: Transport>: Clone {
    /// Creates new API namespace
    fn new(transport: T) -> Self;

    /// Borrows a transport.
    fn transport(&self) -> &T;
}

/// `Web3` wrapper for all namespaces
#[derive(Debug, Clone)]
pub struct Web3<T: Transport> {
    transport: T,
}

impl<T: Transport> Web3<T> {
    /// Create new `Web3` with given transport
    pub fn new(transport: T) -> Self {
        Web3 { transport }
    }

    /// Borrows a transport.
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// set the max response bytes
    pub fn set_max_response_bytes(&mut self, bytes: u64) {
        self.transport.set_max_response_bytes(bytes)
    }

    /// Access methods from custom namespace
    pub fn api<A: Namespace<T>>(&self) -> A {
        A::new(self.transport.clone())
    }

    /// Access methods from `eth` namespace
    pub fn eth(&self) -> eth::Eth<T> {
        self.api()
    }

    /// Call json rpc directly
    pub async fn json_rpc_call(&self, body: &str, options: CallOptions) -> error::Result<String> {
        let request: Call = serde_json::from_str(body).map_err(|_| Error::Decoder(body.to_string()))?;
        // currently, the request id is not used
        self.transport
            .send(RequestId::default(), request, options)
            .await
            .map(|v| format!("{}", v))
    }
}
