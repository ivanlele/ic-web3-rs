//! Ethereum JSON-RPC client (Web3).

#![allow(
    clippy::type_complexity,
    clippy::wrong_self_convention,
    clippy::single_match,
    clippy::let_unit_value,
    clippy::match_wild_err_arm
)]
// #![warn(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]
// select! in WS transport
#![recursion_limit = "256"]

use ic_cdk::api::management_canister::http_request::TransformContext;
use jsonrpc_core as rpc;

/// Re-export of the `futures` crate.
#[macro_use]
pub extern crate futures;
pub use futures::executor::{block_on, block_on_stream};

pub use ethabi;
use transports::ic_http_client::CallOptions;

// it needs to be before other modules
// otherwise the macro for tests is not available.
#[macro_use]
pub mod helpers;

pub mod api;
pub mod contract;
pub mod error;
pub mod ic;
pub mod signing;
pub mod transforms;
pub mod transports;
pub mod types;
// pub mod tx_helpers;

pub use crate::{
    api::Web3,
    error::{Error, Result},
};

/// Assigned RequestId
pub type RequestId = usize;

// TODO [ToDr] The transport most likely don't need to be thread-safe.
// (though it has to be Send)
/// Transport implementation
pub trait Transport: std::fmt::Debug + Clone {
    /// The type of future this transport returns when a call is made.
    type Out: futures::Future<Output = error::Result<rpc::Value>>;

    /// Prepare serializable RPC call for given method with parameters.
    fn prepare(&self, method: &str, params: Vec<rpc::Value>) -> (RequestId, rpc::Call);

    /// Execute prepared RPC call.
    fn send(&self, id: RequestId, request: rpc::Call, options: CallOptions) -> Self::Out;

    /// Execute remote method with given parameters.
    fn execute(&self, method: &str, params: Vec<rpc::Value>, options: CallOptions) -> Self::Out {
        let (id, request) = self.prepare(method, params);
        self.send(id, request, options)
    }

    /// set the max response bytes, do nothing by default
    fn set_max_response_bytes(&mut self, bytes: u64) {}
}

impl<X, T> Transport for X
where
    T: Transport + ?Sized,
    X: std::ops::Deref<Target = T>,
    X: std::fmt::Debug,
    X: Clone,
{
    type Out = T::Out;

    fn prepare(&self, method: &str, params: Vec<rpc::Value>) -> (RequestId, rpc::Call) {
        (**self).prepare(method, params)
    }

    fn send(&self, id: RequestId, request: rpc::Call, options: CallOptions) -> Self::Out {
        (**self).send(id, request, options)
    }
}
