//! Easy to use utilities for confirmations.

use crate::{
    api::{Eth, Namespace},
    error,
    transports::ic_http_client::CallOptions,
    types::{Bytes, TransactionReceipt, TransactionRequest, H256, U64},
    Transport,
};
use futures::{Future, StreamExt};
use std::time::Duration;

/// Checks whether an event has been confirmed.
pub trait ConfirmationCheck {
    /// Future resolved when is known whether an event has been confirmed.
    type Check: Future<Output = error::Result<Option<U64>>>;

    /// Should be called to get future which resolves when confirmation state is known.
    fn check(&self) -> Self::Check;
}

impl<F, T> ConfirmationCheck for F
where
    F: Fn() -> T,
    T: Future<Output = error::Result<Option<U64>>>,
{
    type Check = T;

    fn check(&self) -> Self::Check {
        (*self)()
    }
}

async fn transaction_receipt_block_number_check<T: Transport>(
    eth: &Eth<T>,
    hash: H256,
    options: CallOptions,
) -> error::Result<Option<U64>> {
    let receipt = eth.transaction_receipt(hash, options).await?;
    Ok(receipt.and_then(|receipt| receipt.block_number))
}
