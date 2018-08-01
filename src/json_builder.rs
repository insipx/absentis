use failure::*;
use serde_derive::*;
use serde_json::*;
use crate::types::JSON_RPC_VERSION;

#[derive(Serialize, Deserialize)]
pub struct JsonBuilder {
    id: usize,
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
            id: 1,
            result: None,
            method: None,
            params: Vec::new(), 
        } 
    }
}


impl JsonBuilder {
    pub fn result<T: Into<String>>(&mut self, val: T) -> &mut Self {
        let new = self;
        new.result = Some(val.into());
        new
    }

    pub fn method<T: Into<String>>(&mut self, val: T) -> &mut Self {
        let new = self;
        new.method = Some(val.into());
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

        let json = JsonBuilder::default().method("eth_blockNumber").build().expect("Bulding JSON failed");

        assert_eq!(test, json);
    }
}
