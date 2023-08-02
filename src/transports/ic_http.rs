//! IC HTTP Transport

use crate::transports::ICHttpClient;
use crate::{
    error::{Error, Result, TransportError},
    helpers, RequestId, Transport,
};
#[cfg(not(feature = "wasm"))]
use futures::future::BoxFuture;
use ic_cdk::api::management_canister::http_request::TransformContext;
use jsonrpc_core::types::{Call, Output, Request, Value};
use serde::de::DeserializeOwned;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

pub use super::ic_http_client::{CallOptions, CallOptionsBuilder};

/// HTTP Transport
#[derive(Clone, Debug)]
pub struct ICHttp {
    client: ICHttpClient,
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    url: String,
    id: AtomicUsize,
}

impl ICHttp {
    /// Create new HTTP transport connecting to given URL, cycles: cycles amount to perform http call
    ///
    /// Note that the http [Client] automatically enables some features like setting the basic auth
    /// header or enabling a proxy from the environment. You can customize it with
    /// [Http::with_client].
    pub fn new(url: &str, max_resp: Option<u64>) -> Result<Self> {
        Ok(Self {
            client: ICHttpClient::new(max_resp),
            inner: Arc::new(Inner {
                url: url.to_string(),
                id: AtomicUsize::new(0),
            }),
        })
    }

    fn next_id(&self) -> RequestId {
        self.inner.id.fetch_add(1, Ordering::AcqRel)
    }

    fn new_request(&self) -> (ICHttpClient, String) {
        (self.client.clone(), self.inner.url.clone())
    }
}

// Id is only used for logging.
async fn execute_rpc<T: DeserializeOwned>(
    client: &ICHttpClient,
    url: String,
    request: &Request,
    id: RequestId,
    options: CallOptions,
) -> Result<T> {
    let response = client
        .post(url, request, options)
        .await
        .map_err(|err| Error::Transport(TransportError::Message(err)))?;
    helpers::arbitrary_precision_deserialize_workaround(&response).map_err(|err| {
        Error::Transport(TransportError::Message(format!(
            "failed to deserialize response: {}: {}",
            err,
            String::from_utf8_lossy(&response)
        )))
    })
}

type RpcResult = Result<Value>;

impl Transport for ICHttp {
    type Out = BoxFuture<'static, Result<Value>>;

    fn prepare(&self, method: &str, params: Vec<Value>) -> (RequestId, Call) {
        let id = self.next_id();
        let request = helpers::build_request(id, method, params);
        (id, request)
    }

    fn send(&self, id: RequestId, call: Call, options: CallOptions) -> Self::Out {
        let (client, url) = self.new_request();
        Box::pin(async move {
            let output: Output = execute_rpc(&client, url, &Request::Single(call), id, options).await?;
            helpers::to_result_from_output(output)
        })
    }

    fn set_max_response_bytes(&mut self, v: u64) {
        self.client.set_max_response_bytes(v);
    }
}

fn id_of_output(output: &Output) -> Result<RequestId> {
    let id = match output {
        Output::Success(success) => &success.id,
        Output::Failure(failure) => &failure.id,
    };
    match id {
        jsonrpc_core::Id::Num(num) => Ok(*num as RequestId),
        _ => Err(Error::InvalidResponse("response id is not u64".to_string())),
    }
}
