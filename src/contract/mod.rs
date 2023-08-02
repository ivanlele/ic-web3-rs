//! Ethereum Contract Interface

use crate::{
    api::{Eth, Namespace},
    contract::tokens::{Detokenize, Tokenize},
    futures::Future,
    ic::KeyInfo,
    transports::ic_http_client::CallOptions,
    types::{
        AccessList, Address, BlockId, Bytes, CallRequest, FilterBuilder, TransactionCondition, TransactionParameters,
        TransactionReceipt, TransactionRequest, H256, U256, U64,
    },
    Transport,
};
use std::{collections::HashMap, hash::Hash, time};

mod error;
pub mod tokens;

pub use crate::contract::error::Error;

/// Contract `Result` type.
pub type Result<T> = std::result::Result<T, Error>;

/// Contract Call/Query Options
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Options {
    /// Fixed gas limit
    pub gas: Option<U256>,
    /// Fixed gas price
    pub gas_price: Option<U256>,
    /// Value to transfer
    pub value: Option<U256>,
    /// Fixed transaction nonce
    pub nonce: Option<U256>,
    /// A condition to satisfy before including transaction.
    pub condition: Option<TransactionCondition>,
    /// Transaction type, Some(1) for AccessList transaction, None for Legacy
    pub transaction_type: Option<U64>,
    /// Access list
    pub access_list: Option<AccessList>,
    /// Max fee per gas
    pub max_fee_per_gas: Option<U256>,
    /// miner bribe
    pub max_priority_fee_per_gas: Option<U256>,
    pub call_options: Option<CallOptions>,
}

impl Options {
    /// Create new default `Options` object with some modifications.
    pub fn with<F>(func: F) -> Options
    where
        F: FnOnce(&mut Options),
    {
        let mut options = Options::default();
        func(&mut options);
        options
    }
}

/// Ethereum Contract Interface
#[derive(Debug, Clone)]
pub struct Contract<T: Transport> {
    address: Address,
    eth: Eth<T>,
    abi: ethabi::Contract,
}

impl<T: Transport> Contract<T> {}

impl<T: Transport> Contract<T> {
    /// Creates new Contract Interface given blockchain address and ABI
    pub fn new(eth: Eth<T>, address: Address, abi: ethabi::Contract) -> Self {
        Contract { address, eth, abi }
    }

    /// Creates new Contract Interface given blockchain address and JSON containing ABI
    pub fn from_json(eth: Eth<T>, address: Address, json: &[u8]) -> ethabi::Result<Self> {
        let abi = ethabi::Contract::load(json)?;
        Ok(Self::new(eth, address, abi))
    }

    /// Get the underlying contract ABI.
    pub fn abi(&self) -> &ethabi::Contract {
        &self.abi
    }

    /// Returns contract address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Execute a contract function
    pub async fn call<P>(&self, func: &str, params: P, from: Address, options: Options) -> Result<H256>
    where
        P: Tokenize,
    {
        let data = self.abi.function(func)?.encode_input(&params.into_tokens())?;
        let Options {
            gas,
            gas_price,
            value,
            nonce,
            condition,
            transaction_type,
            access_list,
            max_fee_per_gas,
            max_priority_fee_per_gas,
            call_options,
        } = options;
        self.eth
            .send_transaction(
                TransactionRequest {
                    from,
                    to: Some(self.address),
                    gas,
                    gas_price,
                    value,
                    nonce,
                    data: Some(Bytes(data)),
                    condition,
                    transaction_type,
                    access_list,
                    max_fee_per_gas,
                    max_priority_fee_per_gas,
                },
                call_options.unwrap_or_default(),
            )
            .await
            .map_err(Error::from)
    }

    /// Estimate gas required for this function call.
    pub async fn estimate_gas<P>(&self, func: &str, params: P, from: Address, options: Options) -> Result<U256>
    where
        P: Tokenize,
    {
        let data = self.abi.function(func)?.encode_input(&params.into_tokens())?;
        self.eth
            .estimate_gas(
                CallRequest {
                    from: Some(from),
                    to: Some(self.address),
                    gas: options.gas,
                    gas_price: options.gas_price,
                    value: options.value,
                    data: Some(Bytes(data)),
                    transaction_type: options.transaction_type,
                    access_list: options.access_list,
                    max_fee_per_gas: options.max_fee_per_gas,
                    max_priority_fee_per_gas: options.max_priority_fee_per_gas,
                },
                None,
                options.call_options.unwrap_or_default(),
            )
            .await
            .map_err(Into::into)
    }
    pub async fn _estimate_gas(
        &self,
        from: Address,
        tx: &TransactionParameters,
        call_options: CallOptions,
    ) -> Result<U256> {
        self.eth
            .estimate_gas(
                CallRequest {
                    from: Some(from),
                    to: tx.to,
                    gas: None,
                    gas_price: tx.gas_price,
                    value: Some(tx.value),
                    data: Some(tx.data.clone()),
                    transaction_type: tx.transaction_type,
                    access_list: tx.access_list.clone(),
                    max_fee_per_gas: tx.max_fee_per_gas,
                    max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                },
                None,
                call_options,
            )
            .await
            .map_err(Into::into)
    }

    /// Call constant function
    pub fn query<R, A, B, P>(
        &self,
        func: &str,
        params: P,
        from: A,
        options: Options,
        block: B,
    ) -> impl Future<Output = Result<R>> + '_
    where
        R: Detokenize,
        A: Into<Option<Address>>,
        B: Into<Option<BlockId>>,
        P: Tokenize,
    {
        let result = self
            .abi
            .function(func)
            .and_then(|function| {
                function
                    .encode_input(&params.into_tokens())
                    .map(|call| (call, function))
            })
            .map(|(call, function)| {
                let call_future = self.eth.call(
                    CallRequest {
                        from: from.into(),
                        to: Some(self.address),
                        gas: options.gas,
                        gas_price: options.gas_price,
                        value: options.value,
                        data: Some(Bytes(call)),
                        transaction_type: options.transaction_type,
                        access_list: options.access_list,
                        max_fee_per_gas: options.max_fee_per_gas,
                        max_priority_fee_per_gas: options.max_priority_fee_per_gas,
                    },
                    block.into(),
                    options.call_options.unwrap_or_default(),
                );
                (call_future, function)
            });
        // NOTE for the batch transport to work correctly, we must call `transport.execute` without ever polling the future,
        // hence it cannot be a fully `async` function.
        async {
            let (call_future, function) = result?;
            let bytes = call_future.await?;
            let output = function.decode_output(&bytes.0)?;
            R::from_tokens(output)
        }
    }

    /// Find events matching the topics.
    pub async fn events<A, B, C, R>(
        &self,
        event: &str,
        topic0: A,
        topic1: B,
        topic2: C,
        options: CallOptions,
    ) -> Result<Vec<R>>
    where
        A: Tokenize,
        B: Tokenize,
        C: Tokenize,
        R: Detokenize,
    {
        fn to_topic<A: Tokenize>(x: A) -> ethabi::Topic<ethabi::Token> {
            let tokens = x.into_tokens();
            if tokens.is_empty() {
                ethabi::Topic::Any
            } else {
                tokens.into()
            }
        }

        let res = self.abi.event(event).and_then(|ev| {
            let filter = ev.filter(ethabi::RawTopicFilter {
                topic0: to_topic(topic0),
                topic1: to_topic(topic1),
                topic2: to_topic(topic2),
            })?;
            Ok((ev.clone(), filter))
        });
        let (ev, filter) = match res {
            Ok(x) => x,
            Err(e) => return Err(e.into()),
        };

        let logs = self
            .eth
            .logs(FilterBuilder::default().topic_filter(filter).build(), options)
            .await?;
        logs.into_iter()
            .map(move |l| {
                let log = ev.parse_log(ethabi::RawLog {
                    topics: l.topics,
                    data: l.data.0,
                })?;

                R::from_tokens(log.params.into_iter().map(|x| x.value).collect::<Vec<_>>())
            })
            .collect::<Result<Vec<R>>>()
    }
}
