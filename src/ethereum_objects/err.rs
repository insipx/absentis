use failure::Fail;
use serde_derive::*;
use serde_json::{self, from_str, from_slice, Error as JError, json, json_internal};
use serde::de::{self, Deserializer, Deserialize, Visitor, SeqAccess, MapAccess};

#[derive(Fail, Debug)]
pub struct TypeMismatchError {
    invalid_type: String
}

impl TypeMismatchError {
    crate fn new(err: String) -> Self {
        TypeMismatchError {
            invalid_type: err
        }
    }
}

impl std::fmt::Display for TypeMismatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.invalid_type)
    }
}

#[derive(Fail, Debug)]
pub enum ResponseBuildError {
    #[fail(display = "Error building JSON Object from 'Result'")]
    SerializationError(#[fail(cause)] serde_json::error::Error),
    #[fail(display = "Hyper Error while building Json Response Object")]
    HyperError(#[fail(cause)] hyper::error::Error),
    #[fail(display = "Mismatched types during build")]
    MismatchedTypes(TypeMismatchError)
}

impl From<serde_json::error::Error> for ResponseBuildError {
    fn from(err: serde_json::error::Error) -> Self {
        ResponseBuildError::SerializationError(err)
    }
}

// expects a string and value 
#[macro_export]
macro_rules! mismatched_types {
    ($expected_type: expr, $recvd_type: ident) => ({
        let string = format!("Expected type `{}`, got `{}` in {}", $expected_type, $recvd_type, err_loc!());
        Err(ResponseBuildError::MismatchedTypes(TypeMismatchError::new(string)))
    })
}
