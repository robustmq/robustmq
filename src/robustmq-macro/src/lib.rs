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

mod validate_req;

use proc_macro::TokenStream;

/// todo: Used to call parameter validation of gRPC request structure in tonic framework.
///       This is usually used with [`prost-validate`] crate
#[cfg(feature = "validate-req")]
#[proc_macro_attribute]
pub fn validate_req(attr: TokenStream, input: TokenStream) -> TokenStream {
    let origin = input.clone();

    let arg: validate_req::Arg = match syn::parse(attr) {
        Ok(arg) => arg,
        Err(err) => return compile_err(err),
    };

    let ast_fn = match syn::parse::<syn::ItemFn>(input) {
        Ok(ast_fn) => ast_fn,
        Err(err) => return origin_compile_err(origin, err),
    };

    match validate_req::expanded(arg, ast_fn) {
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

/// Injects a statement at the beginning of an `async` block inside `Box::pin(async move { ... })`.
///
/// This is typically used to insert statements into async code blocks expanded by [`async_trait`] macros.
///
/// # Arguments
/// * `block` - The code block containing the `Box::pin(async move { ... })` expression
/// * `inject_stmt` - The statement to be inserted
///
/// # Returns
/// * `Ok(())` if injection succeeded
/// * `Err(syn::Error)` with detailed error message if the structure doesn't match expectations
///
/// [`async_trait`]: https://docs.rs/async-trait
pub(crate) fn inject_into_async_block(
    block: &mut Box<syn::Block>,
    inject_stmt: syn::Stmt,
) -> syn::Result<()> {
    // Because `#[async_trait]` will only perform ast parsing during the compilation phase.
    // During the coding phase, the IDE is too smart and will expand our macros in advance,
    // resulting in false positives, so here we just return Ok().
    // Don't worry, it will be judged again when the insertion is actually executed.
    let async_stmt = if let Some(async_stmt) = block.stmts.first_mut() {
        async_stmt
    } else {
        return Ok(());
    };
    let pin_call = match async_stmt {
        syn::Stmt::Expr(syn::Expr::Call(call), _) => call,
        _ => {
            return Ok(());
        }
    };

    let async_expr = {
        let pin_call_token = pin_call.clone();
        pin_call.args.first_mut().ok_or_else(|| {
            syn::Error::new_spanned(
                pin_call_token,
                r#""Box::pin()" requires at least one argument"#,
            )
        })?
    };

    if is_box_pin_call(&pin_call.func) {
        if let syn::Expr::Async(syn::ExprAsync {
            block: async_block, ..
        }) = async_expr
        {
            async_block.stmts.insert(0, inject_stmt);
            Ok(())
        } else {
            Err(syn::Error::new_spanned(
                async_expr,
                r#"expected async block inside "Box::pin()""#,
            ))
        }
    } else {
        Ok(())
    }
}

fn is_box_pin_call(expr: &syn::Expr) -> bool {
    if let syn::Expr::Path(syn::ExprPath { path, .. }) = expr {
        path.segments
            .first()
            .map(|seg| seg.ident.eq("Box"))
            .unwrap_or(false)
            && path
                .segments
                .last()
                .map(|seg| seg.ident.eq("pin"))
                .unwrap_or(false)
    } else {
        false
    }
}
