use derive_builder::Builder;
use ic_cdk::api::management_canister::http_request::{HttpResponse, TransformArgs};
use serde_json::Value;

#[derive(Debug, Builder, Default)]
pub struct SingleResultTransformProcessor {
    pub transaction_index: bool,
}

#[derive(Debug, Builder, Default)]
pub struct ArrayResultTransformProcessor {
    pub transaction_index: bool,
    pub log_index: bool,
}

pub trait TransformProcessor {
    fn transform(&self, raw: TransformArgs) -> HttpResponse {
        let mut res = HttpResponse {
            status: raw.response.status.clone(),
            ..Default::default()
        };
        if res.status == 200 {
            res.body = self.process_body(&raw.response.body);
        } else {
            ic_cdk::api::print(format!("Received an error from blockchain: err = {:?}", raw));
        }
        res
    }
    fn process_body(&self, body: &[u8]) -> Vec<u8>;
}

impl TransformProcessor for ArrayResultTransformProcessor {
    fn process_body(&self, body: &[u8]) -> Vec<u8> {
        let mut body: Value = serde_json::from_slice(body).unwrap();
        let elements = body.get_mut("result").unwrap().as_array_mut().unwrap();
        for element in elements.iter_mut() {
            if self.transaction_index {
                element
                    .as_object_mut()
                    .unwrap()
                    .insert("transactionIndex".to_string(), Value::from("0x0"));
            }
            if self.log_index {
                element
                    .as_object_mut()
                    .unwrap()
                    .insert("logIndex".to_string(), Value::from("0x0"));
            }
        }
        serde_json::to_vec(&body).unwrap()
    }
}

impl TransformProcessor for SingleResultTransformProcessor {
    fn process_body(&self, body: &[u8]) -> Vec<u8> {
        let mut body: Value = serde_json::from_slice(body).unwrap();
        if self.transaction_index {
            body.get_mut("result")
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("transactionIndex".to_string(), Value::from("0x0"));
        }
        serde_json::to_vec(&body).unwrap()
    }
}
