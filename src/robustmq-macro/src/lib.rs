// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod parse;
mod tools;
mod validate_req;

use proc_macro::TokenStream;

/// A procedural macro for automatic request validation in Tonic gRPC services.
///
/// This attribute macro can be applied either to a service implementation or individual methods
/// to automatically validate incoming requests using a specified validator.
///
/// *WARN: The usage methods of these two are mutually exclusive!*
///
/// # Usage
///
/// The macro requires a `validator` parameter specifying the validator trait to use:
/// ```ignore
/// #[validate_req(validator = foo::bar::traits)]
/// ```
///
/// ## Service-level Application
///
/// When applied to a service implementation, all methods will automatically validate their requests:
/// ```ignore
/// #[tonic::async_trait]
/// #[validate_req(validator = prost_validate::Validator)]
/// impl Greeter for MyGreeter {
///     // All methods will be validated
/// }
/// ```
///
/// ## Method-level Application
///
/// When applied to individual methods, only those methods will validate their requests:
/// ```ignore
/// #[validate_req(validator = prost_validate::Validator)]
/// async fn say_hello(
///     &self,
///     request: tonic::Request<HelloRequest>,
/// ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
///     // Method implementation
/// }
/// ```
///
/// # Expansion
///
/// The macro expands to validation code that:
/// 1. Extracts the request reference using `get_ref()`
/// 2. Verifies the request implements the validator trait
/// 3. Calls `validate()` on the request
/// 4. Returns an invalid argument status on validation failure
///
///
/// # Requirements
///
/// - The request type must implement the specified validator trait
/// - The validator trait must provide a `validate()` method
/// - The request wrapper must have a `get_ref()` method (standard in [`tonic`] requests)
///
/// # Errors
///
/// Returns `tonic::Status::invalid_argument` with the validation error message
/// if validation fails.
///
/// # Examples
///
/// ```
/// use tonic::{Request, Response, Status};
/// use robustmq_macro::validate_req;
///
/// #[tonic::async_trait]
/// #[validate_req(validator = prost_validate::Validator)]
/// impl Greeter for MyGreeter {
///     async fn say_hello(
///         &self,
///         request: Request<HelloRequest>,
///     ) -> Result<Response<HelloReply>, Status> {
///         Ok(Response::new(HelloReply {
///             message: format!("Hello {}!", request.get_ref().name),
///         }))
///     }
/// }
/// ```
#[cfg(feature = "validate-req")]
#[proc_macro_attribute]
pub fn validate_req(attr: TokenStream, input: TokenStream) -> TokenStream {
    let origin = input.clone();

    let arg: validate_req::Arg = match syn::parse(attr) {
        Ok(arg) => arg,
        Err(err) => return compile_err(err),
    };

    let ast_item: ParseItem = syn::parse_macro_input!(input as ParseItem);

    match validate_req::expanded(arg, ast_item) {
        Ok(expanded) => expanded.into(),
        Err(err) => origin_compile_err(origin, err),
    }
}

/// Convert error to TokenStream
///
/// This is usually used when we want the error to completely override the macro output,
/// and is suitable for serious errors.
#[allow(unused)]
pub(crate) fn compile_err(err: syn::Error) -> TokenStream {
    err.to_compile_error().into()
}

/// Convert errors to TokenStream and shipped to the original TokenStream
///
/// This preserves the original code context information and allows the compiler to
/// continue checking other parts. Even if the macro fails, the compiler can still check
/// other parts of the original code (such as other functions, structures, etc.).
#[allow(unused)]
pub(crate) fn origin_compile_err(mut origin: TokenStream, err: syn::Error) -> TokenStream {
    let compile_error = compile_err(err);
    origin.extend(compile_error);
    origin
}
