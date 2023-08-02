//! Partial implementation of the `Accounts` namespace.

use crate::ic::{ic_raw_sign, recover_address, KeyInfo};
use crate::{api::Namespace, signing, types::H256, Transport};

/// `Accounts` namespace
#[derive(Debug, Clone)]
pub struct Accounts<T> {
    transport: T,
}

impl<T: Transport> Namespace<T> for Accounts<T> {
    fn new(transport: T) -> Self
    where
        Self: Sized,
    {
        Accounts { transport }
    }

    fn transport(&self) -> &T {
        &self.transport
    }
}

impl<T: Transport> Accounts<T> {
    /// Hash a message according to EIP-191.
    ///
    /// The data is a UTF-8 encoded string and will enveloped as follows:
    /// `"\x19Ethereum Signed Message:\n" + message.length + message` and hashed
    /// using keccak256.
    pub fn hash_message<S>(&self, message: S) -> H256
    where
        S: AsRef<[u8]>,
    {
        signing::hash_message(message)
    }
}

// #[cfg(feature = "signing")]
mod accounts_signing {
    use super::*;
    use crate::{
        api::Web3,
        error,
        signing::Signature,
        types::{
            AccessList, Address, Bytes, Recovery, RecoveryMessage, SignedData, SignedTransaction,
            TransactionParameters, U256, U64,
        },
    };
    use rlp::RlpStream;
    // use std::convert::TryInto;

    const LEGACY_TX_ID: u64 = 0;
    const ACCESSLISTS_TX_ID: u64 = 1;
    const EIP1559_TX_ID: u64 = 2;

    impl<T: Transport> Accounts<T> {
        /// Gets the parent `web3` namespace
        fn web3(&self) -> Web3<T> {
            Web3::new(self.transport.clone())
        }
        
        pub async fn sign_transaction(
            &self,
            tx: TransactionParameters,
            from: String,
            key_info: KeyInfo,
            chain_id: u64,
        ) -> error::Result<SignedTransaction> {
            let gas_price = match tx.transaction_type {
                Some(tx_type) if tx_type == U64::from(EIP1559_TX_ID) && tx.max_fee_per_gas.is_some() => {
                    tx.max_fee_per_gas.unwrap()
                }
                _ => tx.gas_price.unwrap(),
            };

            let max_priority_fee_per_gas = match tx.transaction_type {
                Some(tx_type) if tx_type == U64::from(EIP1559_TX_ID) => {
                    tx.max_priority_fee_per_gas.unwrap_or(gas_price)
                }
                _ => gas_price,
            };

            let tx = Transaction {
                to: tx.to,
                nonce: tx.nonce.unwrap(),
                gas: tx.gas,
                gas_price,
                value: tx.value,
                data: tx.data.0,
                transaction_type: tx.transaction_type,
                access_list: tx.access_list.unwrap_or_default(),
                max_priority_fee_per_gas,
            };

            let signed = tx.sign(from, key_info, chain_id).await;
            Ok(signed)
        }
    }
    /// A transaction used for RLP encoding, hashing and signing.
    #[derive(Debug)]
    pub struct Transaction {
        pub to: Option<Address>,
        pub nonce: U256,
        pub gas: U256,
        pub gas_price: U256,
        pub value: U256,
        pub data: Vec<u8>,
        pub transaction_type: Option<U64>,
        pub access_list: AccessList,
        pub max_priority_fee_per_gas: U256,
    }

    impl Transaction {
        fn rlp_append_legacy(&self, stream: &mut RlpStream) {
            stream.append(&self.nonce);
            stream.append(&self.gas_price);
            stream.append(&self.gas);
            if let Some(to) = self.to {
                stream.append(&to);
            } else {
                stream.append(&"");
            }
            stream.append(&self.value);
            stream.append(&self.data);
        }

        fn encode_legacy(&self, chain_id: u64, signature: Option<&Signature>) -> RlpStream {
            let mut stream = RlpStream::new();
            stream.begin_list(9);

            self.rlp_append_legacy(&mut stream);

            if let Some(signature) = signature {
                self.rlp_append_signature(&mut stream, signature);
            } else {
                stream.append(&chain_id);
                stream.append(&0u8);
                stream.append(&0u8);
            }

            stream
        }

        fn encode_access_list_payload(&self, chain_id: u64, signature: Option<&Signature>) -> RlpStream {
            let mut stream = RlpStream::new();

            let list_size = if signature.is_some() { 11 } else { 8 };
            stream.begin_list(list_size);

            // append chain_id. from EIP-2930: chainId is defined to be an integer of arbitrary size.
            stream.append(&chain_id);

            self.rlp_append_legacy(&mut stream);
            self.rlp_append_access_list(&mut stream);

            if let Some(signature) = signature {
                self.rlp_append_signature(&mut stream, signature);
            }

            stream
        }

        fn encode_eip1559_payload(&self, chain_id: u64, signature: Option<&Signature>) -> RlpStream {
            let mut stream = RlpStream::new();

            let list_size = if signature.is_some() { 12 } else { 9 };
            stream.begin_list(list_size);

            // append chain_id. from EIP-2930: chainId is defined to be an integer of arbitrary size.
            stream.append(&chain_id);

            stream.append(&self.nonce);
            stream.append(&self.max_priority_fee_per_gas);
            stream.append(&self.gas_price);
            stream.append(&self.gas);
            if let Some(to) = self.to {
                stream.append(&to);
            } else {
                stream.append(&"");
            }
            stream.append(&self.value);
            stream.append(&self.data);

            self.rlp_append_access_list(&mut stream);

            if let Some(signature) = signature {
                self.rlp_append_signature(&mut stream, signature);
            }

            stream
        }

        fn rlp_append_signature(&self, stream: &mut RlpStream, signature: &Signature) {
            stream.append(&signature.v);
            stream.append(&U256::from_big_endian(signature.r.as_bytes()));
            stream.append(&U256::from_big_endian(signature.s.as_bytes()));
        }

        fn rlp_append_access_list(&self, stream: &mut RlpStream) {
            stream.begin_list(self.access_list.len());
            for access in self.access_list.iter() {
                stream.begin_list(2);
                stream.append(&access.address);
                stream.begin_list(access.storage_keys.len());
                for storage_key in access.storage_keys.iter() {
                    stream.append(storage_key);
                }
            }
        }

        fn encode(&self, chain_id: u64, signature: Option<&Signature>) -> Vec<u8> {
            match self.transaction_type.map(|t| t.as_u64()) {
                Some(LEGACY_TX_ID) | None => {
                    let stream = self.encode_legacy(chain_id, signature);
                    stream.out().to_vec()
                }

                Some(ACCESSLISTS_TX_ID) => {
                    let tx_id: u8 = ACCESSLISTS_TX_ID as u8;
                    let stream = self.encode_access_list_payload(chain_id, signature);
                    [&[tx_id], stream.as_raw()].concat()
                }

                Some(EIP1559_TX_ID) => {
                    let tx_id: u8 = EIP1559_TX_ID as u8;
                    let stream = self.encode_eip1559_payload(chain_id, signature);
                    [&[tx_id], stream.as_raw()].concat()
                }

                _ => {
                    panic!("Unsupported transaction type");
                }
            }
        }

        pub async fn sign(self, from: String, key_info: KeyInfo, chain_id: u64) -> SignedTransaction {
            let adjust_v_value = matches!(self.transaction_type.map(|t| t.as_u64()), Some(LEGACY_TX_ID) | None);

            let encoded = self.encode(chain_id, None);

            let hash = signing::keccak256(encoded.as_ref());

            let res = match ic_raw_sign(hash.to_vec(), key_info).await {
                Ok(v) => v,
                Err(e) => {
                    panic!("{}", e);
                }
            };

            let v = if from.contains(&recover_address(hash.clone().to_vec(), res.clone(), 0)) {
                if adjust_v_value {
                    2 * chain_id + 35 + 0
                } else {
                    0
                }
            } else {
                if adjust_v_value {
                    2 * chain_id + 35 + 1
                } else {
                    1
                }
            };

            let r_arr = H256::from_slice(&res[0..32]);
            let s_arr = H256::from_slice(&res[32..64]);
            let sig = Signature {
                v: v.clone(),
                r: r_arr.clone().into(),
                s: s_arr.clone().into(),
            };

            let signed = self.encode(chain_id, Some(&sig));
            let transaction_hash = signing::keccak256(signed.as_ref()).into();

            SignedTransaction {
                message_hash: hash.into(),
                v,
                r: r_arr.into(),
                s: s_arr.into(),
                raw_transaction: signed.into(),
                transaction_hash,
            }
        }
    }
}

