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

use crate::parse::ParseItem::{Fn, Impl};
use syn::parse::ParseStream;
use syn::Token;

pub(crate) enum ParseItem {
    Fn(syn::ItemFn),
    Impl(syn::ItemImpl),
}

impl syn::parse::Parse for ParseItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let outer_attrs = input.call(syn::Attribute::parse_outer)?;

        // NOTE: Since the `unsafe` token writing of impl and fn is inconsistent,
        // and fn supports pub Visibility, we need to parse in the following order
        let mut lookahead = input.lookahead1();
        if lookahead.peek(Token![pub]) {
            input.parse::<Token![pub]>()?;
            lookahead = input.lookahead1();
        }
        if lookahead.peek(Token![unsafe]) {
            input.parse::<Token![unsafe]>()?;
            lookahead = input.lookahead1();
        }

        if lookahead.peek(Token![impl]) {
            let mut item_impl = input.parse::<syn::ItemImpl>()?;
            item_impl.attrs = outer_attrs;
            Ok(Impl(item_impl))
        } else if lookahead.peek(Token![fn]) {
            let mut item_fn = input.parse::<syn::ItemFn>()?;
            item_fn.attrs = outer_attrs;
            Ok(Fn(item_fn))
        } else {
            Err(lookahead.error())
        }
    }
}
