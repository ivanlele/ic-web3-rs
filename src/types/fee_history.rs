use crate::types::{BlockNumber, U256};
use serde::{Deserialize, Serialize};

/// The fee history type returned from `eth_feeHistory` call.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeHistory {
    /// Lowest number block of the returned range.
    pub oldest_block: BlockNumber,
    /// A vector of block base fees per gas. This includes the next block after the newest of the returned range, because this value can be derived from the newest block. Zeroes are returned for pre-EIP-1559 blocks.
    pub base_fee_per_gas: Vec<U256>,
    /// A vector of block gas used ratios. These are calculated as the ratio of gas used and gas limit.
    pub gas_used_ratio: Vec<f64>,
    /// A vector of effective priority fee per gas data points from a single block. All zeroes are returned if the block is empty. Returned only if requested.
    pub reward: Option<Vec<Vec<U256>>>,
}
