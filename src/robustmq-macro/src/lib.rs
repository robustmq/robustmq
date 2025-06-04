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

use proc_macro::TokenStream;
use quote::quote;

/// todo: Used to call parameter validation of gRPC request structure in tonic framework.
///       This is usually used with [`prost-validate`] crate
#[cfg(feature = "validate-req")]
#[proc_macro_attribute]
pub fn validate_req(_attr: TokenStream, _input: TokenStream) -> TokenStream {
    quote! {}.into()
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
