//! `Web3` implementation

mod accounts;
mod eth;
mod eth_subscribe;
mod net;
mod parity;
mod parity_accounts;
mod parity_set;
mod personal;
mod traces;
mod txpool;
mod web3;

pub use self::{
    accounts::Accounts,
    eth::Eth,
    eth_subscribe::{EthSubscribe, SubscriptionId, SubscriptionStream},
    net::Net,
    parity::Parity,
    parity_accounts::ParityAccounts,
    parity_set::ParitySet,
    personal::Personal,
    traces::Traces,
    txpool::Txpool,
    web3::Web3 as Web3Api,
};

use crate::{
    confirm, error,
    transports::ic_http_client::CallOptions,
    types::{Bytes, TransactionReceipt, TransactionRequest, U64},
    DuplexTransport, Error, RequestId, Transport,
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

    /// Access methods from `accounts` namespace
    pub fn accounts(&self) -> accounts::Accounts<T> {
        self.api()
    }

    /// Access methods from `eth` namespace
    pub fn eth(&self) -> eth::Eth<T> {
        self.api()
    }

    /// Access methods from `net` namespace
    pub fn net(&self) -> net::Net<T> {
        self.api()
    }

    /// Access methods from `web3` namespace
    pub fn web3(&self) -> web3::Web3<T> {
        self.api()
    }

    /// Access methods from `parity` namespace
    pub fn parity(&self) -> parity::Parity<T> {
        self.api()
    }

    /// Access methods from `parity_accounts` namespace
    pub fn parity_accounts(&self) -> parity_accounts::ParityAccounts<T> {
        self.api()
    }

    /// Access methods from `parity_set` namespace
    pub fn parity_set(&self) -> parity_set::ParitySet<T> {
        self.api()
    }

    /// Access methods from `personal` namespace
    pub fn personal(&self) -> personal::Personal<T> {
        self.api()
    }

    /// Access methods from `trace` namespace
    pub fn trace(&self) -> traces::Traces<T> {
        self.api()
    }

    /// Access methods from `txpool` namespace
    pub fn txpool(&self) -> txpool::Txpool<T> {
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

impl<T: DuplexTransport> Web3<T> {
    /// Access subscribe methods from `eth` namespace
    pub fn eth_subscribe(&self) -> eth_subscribe::EthSubscribe<T> {
        self.api()
    }
}
