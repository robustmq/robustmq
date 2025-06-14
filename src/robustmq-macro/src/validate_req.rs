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

use crate::parse::ParseItem;
use crate::tools;
use crate::tools::inject_into_async_block;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::{parse_quote, ImplItem};

pub struct Arg {
    #[allow(dead_code)]
    pub(crate) validator_path: syn::ExprPath,
}

impl syn::parse::Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                r#"must provide the "validator" parameter to specify the Validator trait to implement"#,
            ));
        }

        let meta = input.parse::<syn::MetaNameValue>().map_err(|mut err| {
            err.combine(syn::Error::new(
                err.span(),
                r#"invalid validator definition, expected "validator = example::Validator""#,
            ));
            err
        })?;

        let validator_path = if meta.path.is_ident("validator") {
            if let syn::Expr::Path(validator_path) = meta.value {
                Ok(validator_path)
            } else {
                Err(syn::Error::new(
                    meta.span(),
                    r#"expected a path like "std::mem::replace" possibly containing generic parameters and a qualified self-type."#,
                ))
            }
        } else {
            Err(syn::Error::new(
                input.span(),
                r#"invalid attribute parameter definition, expected "validator""#,
            ))
        }?;

        Ok(Self { validator_path })
    }
}

pub(super) fn expanded(_arg: Arg, ast_item: ParseItem) -> syn::Result<TokenStream> {
    match ast_item {
        ParseItem::Fn(mut ast_fn) => {
            let (first_arg, _first_arg_ty) = tools::extract_first_non_self_arg(&ast_fn.sig)?;
            // todo: for more robust judgment, add generic feature constraints

            let validate_stmt = expand_stmt(first_arg);
            inject_into_async_block(&mut ast_fn.block, validate_stmt)?;

            Ok(quote! {#ast_fn})
        }
        ParseItem::Impl(mut ast_impl) => {
            for item in &mut ast_impl.items {
                #[allow(clippy::single_match)]
                match item {
                    ImplItem::Fn(ref mut ast_fn) => {
                        let (first_arg, _first_arg_ty) =
                            tools::extract_first_non_self_arg(&ast_fn.sig)?;
                        // todo: for more robust judgment, add generic feature constraints

                        let validate_stmt = expand_stmt(first_arg);
                        inject_into_async_block(&mut ast_fn.block, validate_stmt)?;
                    }
                    _ => {}
                }
            }
            Ok(quote! {#ast_impl})
        }
    }
}

fn expand_stmt(arg: syn::Ident) -> syn::Stmt {
    let __req_validate_ref = format_ident!("__{}__validate_ref", arg);
    parse_quote!({
        {
            let #__req_validate_ref = #arg.get_ref();
            if let Err(e) = #__req_validate_ref.validate() {
                return Err(tonic::Status::invalid_argument(e.to_string()));
            }
        }
    })
}
