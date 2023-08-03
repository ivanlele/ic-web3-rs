//! `Eth` namespace

use crate::{
    api::Namespace,
    helpers::{self, CallFuture},
    transports::ic_http_client::CallOptions,
    types::{
        Address, Block, BlockHeader, BlockId, BlockNumber, Bytes, CallRequest, FeeHistory, Filter, Index, Log, Proof,
        Transaction, TransactionId, TransactionReceipt, TransactionRequest, Work, H256, H520, H64, U256,
        U64,
    },
    Transport,
};

/// `Eth` namespace
#[derive(Debug, Clone)]
pub struct Eth<T> {
    transport: T,
}

impl<T: Transport> Namespace<T> for Eth<T> {
    fn new(transport: T) -> Self
    where
        Self: Sized,
    {
        Eth { transport }
    }

    fn transport(&self) -> &T {
        &self.transport
    }
}

impl<T: Transport> Eth<T> {
    /// Get list of available accounts.
    pub fn accounts(&self, options: CallOptions) -> CallFuture<Vec<Address>, T::Out> {
        CallFuture::new(self.transport.execute("eth_accounts", vec![], options))
    }

    /// Get current block number
    pub fn block_number(&self, options: CallOptions) -> CallFuture<U64, T::Out> {
        CallFuture::new(self.transport.execute("eth_blockNumber", vec![], options))
    }

    /// Call a constant method of contract without changing the state of the blockchain.
    pub fn call(&self, req: CallRequest, block: Option<BlockId>, options: CallOptions) -> CallFuture<Bytes, T::Out> {
        let req = helpers::serialize(&req);
        let block = helpers::serialize(&block.unwrap_or_else(|| BlockNumber::Latest.into()));

        CallFuture::new(self.transport.execute("eth_call", vec![req, block], options))
    }

    /// Get coinbase address
    pub fn coinbase(&self, options: CallOptions) -> CallFuture<Address, T::Out> {
        CallFuture::new(self.transport.execute("eth_coinbase", vec![], options))
    }

    /// Compile LLL
    pub fn compile_lll(&self, code: String, options: CallOptions) -> CallFuture<Bytes, T::Out> {
        let code = helpers::serialize(&code);
        CallFuture::new(self.transport.execute("eth_compileLLL", vec![code], options))
    }

    /// Compile Solidity
    pub fn compile_solidity(&self, code: String, options: CallOptions) -> CallFuture<Bytes, T::Out> {
        let code = helpers::serialize(&code);
        CallFuture::new(self.transport.execute("eth_compileSolidity", vec![code], options))
    }

    /// Compile Serpent
    pub fn compile_serpent(&self, code: String, options: CallOptions) -> CallFuture<Bytes, T::Out> {
        let code = helpers::serialize(&code);
        CallFuture::new(self.transport.execute("eth_compileSerpent", vec![code], options))
    }

    /// Call a contract without changing the state of the blockchain to estimate gas usage.
    pub fn estimate_gas(
        &self,
        req: CallRequest,
        block: Option<BlockNumber>,
        options: CallOptions,
    ) -> CallFuture<U256, T::Out> {
        let req = helpers::serialize(&req);

        let args = match block {
            Some(block) => vec![req, helpers::serialize(&block)],
            None => vec![req],
        };

        CallFuture::new(self.transport.execute("eth_estimateGas", args, options))
    }

    /// Get current recommended gas price
    pub fn gas_price(&self, options: CallOptions) -> CallFuture<U256, T::Out> {
        CallFuture::new(self.transport.execute("eth_gasPrice", vec![], options))
    }

    /// Returns a collection of historical gas information. This can be used for evaluating the max_fee_per_gas
    /// and max_priority_fee_per_gas to send the future transactions.
    pub fn fee_history(
        &self,
        block_count: U256,
        newest_block: BlockNumber,
        reward_percentiles: Option<Vec<f64>>,
        options: CallOptions,
    ) -> CallFuture<FeeHistory, T::Out> {
        let block_count = helpers::serialize(&block_count);
        let newest_block = helpers::serialize(&newest_block);
        let reward_percentiles = helpers::serialize(&reward_percentiles);

        CallFuture::new(self.transport.execute(
            "eth_feeHistory",
            vec![block_count, newest_block, reward_percentiles],
            options,
        ))
    }

    /// Get balance of given address
    pub fn balance(
        &self,
        address: Address,
        block: Option<BlockNumber>,
        options: CallOptions,
    ) -> CallFuture<U256, T::Out> {
        let address = helpers::serialize(&address);
        let block = helpers::serialize(&block.unwrap_or(BlockNumber::Latest));

        CallFuture::new(self.transport.execute("eth_getBalance", vec![address, block], options))
    }

    /// Get all logs matching a given filter object
    pub fn logs(&self, filter: Filter, options: CallOptions) -> CallFuture<Vec<Log>, T::Out> {
        let filter = helpers::serialize(&filter);
        CallFuture::new(self.transport.execute("eth_getLogs", vec![filter], options))
    }

    /// Get block details with transaction hashes.
    pub fn block(&self, block: BlockId, options: CallOptions) -> CallFuture<Option<Block<H256>>, T::Out> {
        let include_txs = helpers::serialize(&false);

        let result = match block {
            BlockId::Hash(hash) => {
                let hash = helpers::serialize(&hash);
                self.transport
                    .execute("eth_getBlockByHash", vec![hash, include_txs], options)
            }
            BlockId::Number(num) => {
                let num = helpers::serialize(&num);
                self.transport
                    .execute("eth_getBlockByNumber", vec![num, include_txs], options)
            }
        };

        CallFuture::new(result)
    }

    /// Get block details with full transaction objects.
    pub fn block_with_txs(
        &self,
        block: BlockId,
        options: CallOptions,
    ) -> CallFuture<Option<Block<Transaction>>, T::Out> {
        let include_txs = helpers::serialize(&true);

        let result = match block {
            BlockId::Hash(hash) => {
                let hash = helpers::serialize(&hash);
                self.transport
                    .execute("eth_getBlockByHash", vec![hash, include_txs], options)
            }
            BlockId::Number(num) => {
                let num = helpers::serialize(&num);
                self.transport
                    .execute("eth_getBlockByNumber", vec![num, include_txs], options)
            }
        };

        CallFuture::new(result)
    }

    /// Get number of transactions in block
    pub fn block_transaction_count(&self, block: BlockId, options: CallOptions) -> CallFuture<Option<U256>, T::Out> {
        let result = match block {
            BlockId::Hash(hash) => {
                let hash = helpers::serialize(&hash);
                self.transport
                    .execute("eth_getBlockTransactionCountByHash", vec![hash], options)
            }
            BlockId::Number(num) => {
                let num = helpers::serialize(&num);
                self.transport
                    .execute("eth_getBlockTransactionCountByNumber", vec![num], options)
            }
        };

        CallFuture::new(result)
    }

    /// Get code under given address
    pub fn code(
        &self,
        address: Address,
        block: Option<BlockNumber>,
        options: CallOptions,
    ) -> CallFuture<Bytes, T::Out> {
        let address = helpers::serialize(&address);
        let block = helpers::serialize(&block.unwrap_or(BlockNumber::Latest));

        CallFuture::new(self.transport.execute("eth_getCode", vec![address, block], options))
    }

    /// Get supported compilers
    pub fn compilers(&self, options: CallOptions) -> CallFuture<Vec<String>, T::Out> {
        CallFuture::new(self.transport.execute("eth_getCompilers", vec![], options))
    }

    /// Get chain id
    pub fn chain_id(&self, options: CallOptions) -> CallFuture<U256, T::Out> {
        CallFuture::new(self.transport.execute("eth_chainId", vec![], options))
    }

    /// Get available user accounts. This method is only available in the browser. With MetaMask,
    /// this will cause the popup that prompts the user to allow or deny access to their accounts
    /// to your app.
    pub fn request_accounts(&self, options: CallOptions) -> CallFuture<Vec<Address>, T::Out> {
        CallFuture::new(self.transport.execute("eth_requestAccounts", vec![], options))
    }

    /// Get storage entry
    pub fn storage(
        &self,
        address: Address,
        idx: U256,
        block: Option<BlockNumber>,
        options: CallOptions,
    ) -> CallFuture<H256, T::Out> {
        let address = helpers::serialize(&address);
        let idx = helpers::serialize(&idx);
        let block = helpers::serialize(&block.unwrap_or(BlockNumber::Latest));

        CallFuture::new(
            self.transport
                .execute("eth_getStorageAt", vec![address, idx, block], options),
        )
    }

    /// Get nonce
    pub fn transaction_count(
        &self,
        address: Address,
        block: Option<BlockNumber>,
        options: CallOptions,
    ) -> CallFuture<U256, T::Out> {
        let address = helpers::serialize(&address);
        let block = helpers::serialize(&block.unwrap_or(BlockNumber::Latest));

        CallFuture::new(
            self.transport
                .execute("eth_getTransactionCount", vec![address, block], options),
        )
    }

    /// Get transaction
    pub fn transaction(&self, id: TransactionId, options: CallOptions) -> CallFuture<Option<Transaction>, T::Out> {
        let result = match id {
            TransactionId::Hash(hash) => {
                let hash = helpers::serialize(&hash);
                self.transport.execute("eth_getTransactionByHash", vec![hash], options)
            }
            TransactionId::Block(BlockId::Hash(hash), index) => {
                let hash = helpers::serialize(&hash);
                let idx = helpers::serialize(&index);
                self.transport
                    .execute("eth_getTransactionByBlockHashAndIndex", vec![hash, idx], options)
            }
            TransactionId::Block(BlockId::Number(number), index) => {
                let number = helpers::serialize(&number);
                let idx = helpers::serialize(&index);
                self.transport
                    .execute("eth_getTransactionByBlockNumberAndIndex", vec![number, idx], options)
            }
        };

        CallFuture::new(result)
    }

    /// Get transaction receipt
    pub fn transaction_receipt(
        &self,
        hash: H256,
        options: CallOptions,
    ) -> CallFuture<Option<TransactionReceipt>, T::Out> {
        let hash = helpers::serialize(&hash);

        CallFuture::new(self.transport.execute("eth_getTransactionReceipt", vec![hash], options))
    }

    /// Get uncle header by block ID and uncle index.
    ///
    /// This method is meant for TurboGeth compatiblity,
    /// which is missing transaction hashes in the response.
    pub fn uncle_header(
        &self,
        block: BlockId,
        index: Index,
        options: CallOptions,
    ) -> CallFuture<Option<BlockHeader>, T::Out> {
        self.fetch_uncle(block, index, options)
    }

    /// Get uncle by block ID and uncle index -- transactions only has hashes.
    pub fn uncle(&self, block: BlockId, index: Index, options: CallOptions) -> CallFuture<Option<Block<H256>>, T::Out> {
        self.fetch_uncle(block, index, options)
    }

    fn fetch_uncle<X>(&self, block: BlockId, index: Index, options: CallOptions) -> CallFuture<Option<X>, T::Out> {
        let index = helpers::serialize(&index);

        let result = match block {
            BlockId::Hash(hash) => {
                let hash = helpers::serialize(&hash);
                self.transport
                    .execute("eth_getUncleByBlockHashAndIndex", vec![hash, index], options)
            }
            BlockId::Number(num) => {
                let num = helpers::serialize(&num);
                self.transport
                    .execute("eth_getUncleByBlockNumberAndIndex", vec![num, index], options)
            }
        };

        CallFuture::new(result)
    }

    /// Get uncle count in block
    pub fn uncle_count(&self, block: BlockId, options: CallOptions) -> CallFuture<Option<U256>, T::Out> {
        let result = match block {
            BlockId::Hash(hash) => {
                let hash = helpers::serialize(&hash);
                self.transport
                    .execute("eth_getUncleCountByBlockHash", vec![hash], options)
            }
            BlockId::Number(num) => {
                let num = helpers::serialize(&num);
                self.transport
                    .execute("eth_getUncleCountByBlockNumber", vec![num], options)
            }
        };

        CallFuture::new(result)
    }

    /// Get work package
    pub fn work(&self, options: CallOptions) -> CallFuture<Work, T::Out> {
        CallFuture::new(self.transport.execute("eth_getWork", vec![], options))
    }

    /// Get hash rate
    pub fn hashrate(&self, options: CallOptions) -> CallFuture<U256, T::Out> {
        CallFuture::new(self.transport.execute("eth_hashrate", vec![], options))
    }

    /// Get mining status
    pub fn mining(&self, options: CallOptions) -> CallFuture<bool, T::Out> {
        CallFuture::new(self.transport.execute("eth_mining", vec![], options))
    }

    /// Start new block filter
    pub fn new_block_filter(&self, options: CallOptions) -> CallFuture<U256, T::Out> {
        CallFuture::new(self.transport.execute("eth_newBlockFilter", vec![], options))
    }

    /// Start new pending transaction filter
    pub fn new_pending_transaction_filter(&self, options: CallOptions) -> CallFuture<U256, T::Out> {
        CallFuture::new(
            self.transport
                .execute("eth_newPendingTransactionFilter", vec![], options),
        )
    }

    /// Start new pending transaction filter
    pub fn protocol_version(&self, options: CallOptions) -> CallFuture<String, T::Out> {
        CallFuture::new(self.transport.execute("eth_protocolVersion", vec![], options))
    }

    /// Sends a rlp-encoded signed transaction
    pub fn send_raw_transaction(&self, rlp: Bytes, options: CallOptions) -> CallFuture<H256, T::Out> {
        let rlp = helpers::serialize(&rlp);
        CallFuture::new(self.transport.execute("eth_sendRawTransaction", vec![rlp], options))
    }

    /// Sends a transaction transaction
    pub fn send_transaction(&self, tx: TransactionRequest, options: CallOptions) -> CallFuture<H256, T::Out> {
        let tx = helpers::serialize(&tx);
        CallFuture::new(self.transport.execute("eth_sendTransaction", vec![tx], options))
    }

    /// Signs a hash of given data
    pub fn sign(&self, address: Address, data: Bytes, options: CallOptions) -> CallFuture<H520, T::Out> {
        let address = helpers::serialize(&address);
        let data = helpers::serialize(&data);
        CallFuture::new(self.transport.execute("eth_sign", vec![address, data], options))
    }

    /// Submit hashrate of external miner
    pub fn submit_hashrate(&self, rate: U256, id: H256, options: CallOptions) -> CallFuture<bool, T::Out> {
        let rate = helpers::serialize(&rate);
        let id = helpers::serialize(&id);
        CallFuture::new(self.transport.execute("eth_submitHashrate", vec![rate, id], options))
    }

    /// Submit work of external miner
    pub fn submit_work(
        &self,
        nonce: H64,
        pow_hash: H256,
        mix_hash: H256,
        options: CallOptions,
    ) -> CallFuture<bool, T::Out> {
        let nonce = helpers::serialize(&nonce);
        let pow_hash = helpers::serialize(&pow_hash);
        let mix_hash = helpers::serialize(&mix_hash);
        CallFuture::new(
            self.transport
                .execute("eth_submitWork", vec![nonce, pow_hash, mix_hash], options),
        )
    }

    /// Returns the account- and storage-values of the specified account including the Merkle-proof.
    pub fn proof(
        &self,
        address: Address,
        keys: Vec<U256>,
        block: Option<BlockNumber>,
        options: CallOptions,
    ) -> CallFuture<Option<Proof>, T::Out> {
        let add = helpers::serialize(&address);
        let ks = helpers::serialize(&keys);
        let blk = helpers::serialize(&block.unwrap_or(BlockNumber::Latest));
        CallFuture::new(self.transport.execute("eth_getProof", vec![add, ks, blk], options))
    }
}
