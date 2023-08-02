//! Web3 Types

mod block;
mod bytes;
mod bytes_array;
mod fee_history;
mod log;
mod proof;
mod recovery;
mod signed;
mod transaction;
mod transaction_id;
mod transaction_request;
mod uint;
mod work;

pub use self::{
    block::{Block, BlockHeader, BlockId, BlockNumber},
    bytes::Bytes,
    bytes_array::BytesArray,
    fee_history::FeeHistory,
    log::{Filter, FilterBuilder, Log},
    proof::Proof,
    recovery::{ParseSignatureError, Recovery, RecoveryMessage},
    signed::{SignedData, SignedTransaction, TransactionParameters},
    transaction::{AccessList, AccessListItem, RawTransaction, Receipt as TransactionReceipt, Transaction},
    transaction_id::TransactionId,
    transaction_request::{CallRequest, TransactionCondition, TransactionRequest},
    uint::{H128, H160, H2048, H256, H512, H520, H64, U128, U256, U64},
    work::Work,
};

/// Address
pub type Address = H160;
/// Index in block
pub type Index = U64;
