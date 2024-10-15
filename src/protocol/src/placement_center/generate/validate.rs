use tonic::Status;
use validator::{Validate, ValidationErrors};

pub trait ValidateExt {
    fn validate_ext(&self) -> Result<(), Status>;
}

impl<T: Validate> ValidateExt for T {
    fn validate_ext(&self) -> Result<(), Status> {
        self.validate().map_err(|e: ValidationErrors| {
            Status::invalid_argument(format!("Validation error: {}", e))
        })
    }
}
