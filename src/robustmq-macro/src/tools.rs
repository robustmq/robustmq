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

use syn::spanned::Spanned;

/// Extracts the first non-receiver argument from a function signature.
///
/// This function scans the function's signature and returns the identifier and type
/// of the first argument that is not `self`. It only supports simple identifier patterns
/// like `arg: Type`.
///
/// # Errors
///
/// Returns a `syn::Error` if:
/// - There are no arguments other than `self`.
/// - The first non-`self` argument is not a simple identifier pattern.
///
/// # Example
///
/// ```rust
/// // For a function signature like:
/// struct Foo;
///
/// impl Foo{
///     // This function will return ("value", i32)
///     fn do_something(&self, value: i32){}
/// }
/// ```
pub(crate) fn extract_first_non_self_arg(
    sig: &syn::Signature,
) -> syn::Result<(syn::Ident, &syn::Type)> {
    for input in &sig.inputs {
        match input {
            syn::FnArg::Receiver(_) => continue,
            syn::FnArg::Typed(pat_ty) => {
                return if let syn::Pat::Ident(syn::PatIdent { ident, .. }) = &*pat_ty.pat {
                    Ok((ident.clone(), &pat_ty.ty))
                } else {
                    Err(syn::Error::new(
                        pat_ty.pat.span(),
                        "expected the argument to be an identifier (e.g. `arg: Type`)",
                    ))
                }
            }
        }
    }

    Err(syn::Error::new(
        sig.span(),
        "expected at least one argument after `self`",
    ))
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
    block: &mut syn::Block,
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

/// Checks whether the given expression is a path call to `Box::pin`.
///
/// This function inspects the provided `syn::Expr` to determine if it represents
/// a path expression equivalent to `Box::pin`, such as:
///
/// ```rust
/// Box::pin(async {  });
/// ```
///
/// It does so by verifying that:
/// - The expression is a `syn::Expr::Path`
/// - The first path segment is `Box`
/// - The last path segment is `pin`
///
/// This check does **not** verify the full path structure in between segments,
/// so it may return true for paths like `my::Box::pin`.
///
/// # Arguments
///
/// * `expr` - A reference to a `syn::Expr` to analyze.
///
/// # Returns
///
/// * `true` if the expression is syntactically similar to `Box::pin(...)`
/// * `false` otherwise
pub(crate) fn is_box_pin_call(expr: &syn::Expr) -> bool {
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
