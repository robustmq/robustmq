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

use crate::inject_into_async_block;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::ParseStream;
use syn::parse_quote;
use syn::spanned::Spanned;

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

pub(super) fn expanded(_arg: Arg, mut ast_fn: syn::ItemFn) -> syn::Result<TokenStream> {
    let (first_arg, _first_arg_ty) = ast_fn
        .sig
        .inputs
        .iter()
        .find(|fn_arg| !matches!(fn_arg, syn::FnArg::Receiver(_)))
        .and_then(|fn_arg| match fn_arg {
            syn::FnArg::Typed(pat_ty) => match &*pat_ty.pat {
                syn::Pat::Ident(syn::PatIdent { ident, .. }) => Some((ident, &pat_ty.ty)),
                _ => None,
            },
            _ => None, // this will not be executed here
        })
        .ok_or(syn::Error::new(
            Span::call_site(),
            "method argument must be a plain identifier",
        ))?;

    let __req_validate_ref = format_ident!("__{}__validate_ref_ref", first_arg);
    let validate_stmt = parse_quote!({
        {
            let #__req_validate_ref = #first_arg.get_ref();
            if let Err(e) = #__req_validate_ref.validate() {
                return Err(tonic::Status::invalid_argument(e.to_string()));
            }
        }
    });
    inject_into_async_block(&mut ast_fn.block, validate_stmt)?;

    Ok(quote! {#ast_fn})
}
