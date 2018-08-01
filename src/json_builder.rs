use failure::*;
use serde_derive::*;
use serde_json::*;
use crate::types::*;

#[derive(Serialize, Deserialize)]
pub struct JsonBuilder {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<usize>,
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,
    params: Vec<Value>
}


#[derive(Fail, Debug)]
pub enum JsonBuildError {
    #[fail(display = "Error building JSON Object")]
    SerializationError(#[fail(cause)] serde_json::error::Error)
}

impl From<serde_json::error::Error> for JsonBuildError {
    fn from(err: serde_json::error::Error) -> JsonBuildError {
        JsonBuildError::SerializationError(err)
    }
}

impl Default for JsonBuilder {
    fn default() -> JsonBuilder {
        JsonBuilder {
            jsonrpc: JSON_RPC_VERSION.to_string(),
            id: None,
            result: None,
            method: None,
            params: Vec::new(), 
        } 
    }
}


impl JsonBuilder {

    pub fn method(&mut self, val: ApiCall) -> &mut Self {
        let new = self;
        let (id, method) = val.method_info();
        new.id = Some(id);
        new.method = Some(method.into());
        new
    }

    pub fn params<T: Into<Vec<Value>>>(&mut self, val: T) -> &mut Self {
        let new = self;
        new.params = val.into();
        new
    }

    pub fn build(&self) -> std::result::Result<String, JsonBuildError> {
        Ok(serde_json::to_string(self)?)
    }
}

impl JsonBuilder {
    crate fn get_id(&self) -> usize {
        self.id.expect("Should only be used by `Response Object`")
    }

    crate fn get_result(&self) -> String {
        self.result.expect("Should never be used before a request")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_should_create_json() {
        
        let test = json!({
           "id": 1,
           "jsonrpc": "2.0",
           "method": "eth_blockNumber",
           "params": [],
        }).to_string();

        let json = JsonBuilder::default().method(ApiCall::EthBlockNumber).build().expect("Bulding JSON failed");

        assert_eq!(test, json);
    }
}
