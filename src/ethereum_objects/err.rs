use failure::Fail;

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

