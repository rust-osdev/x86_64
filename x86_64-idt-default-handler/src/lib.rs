use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Expr, ExprLit, ExprRange, ExprReference, Ident, Lit, RangeLimits, Token,
};

extern crate proc_macro;

use proc_macro::TokenStream;

struct DefaultHandler {
    idt: ExprReference,
    function: Ident,
    range: ExprRange,
}

impl Parse for DefaultHandler {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let idt = input
            .parse()
            .map_err(|err| syn::Error::new(err.span(), "expected a `&mut` reference to an IDT"))?;
        input.parse::<Token![,]>()?;
        let function = input.parse().map_err(|err| {
            syn::Error::new(err.span(), "expected the name of a handler function")
        })?;
        input.parse::<Token![,]>()?;
        let range = input.parse()?;
        Ok(DefaultHandler {
            idt,
            function,
            range,
        })
    }
}

#[proc_macro]
pub fn set_default_handler(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DefaultHandler);

    set_default_handler_impl(&input).unwrap_or_else(|err| err.to_compile_error().into())
}

fn set_default_handler_impl(input: &DefaultHandler) -> syn::Result<TokenStream> {
    let DefaultHandler {
        idt,
        function,
        range,
    } = input;

    if idt.mutability.is_none() {
        return Err(syn::Error::new_spanned(
            idt,
            "Must be a `&mut` reference to an IDT",
        ));
    }

    let range_start: u8 = match range.from.as_deref() {
        Some(&Expr::Lit(ExprLit {
            lit: Lit::Int(ref int),
            ..
        })) => int.base10_parse()?,
        Some(other) => return Err(syn::Error::new_spanned(other, "Invalid range start")),
        None => 0,
    };
    let mut range_end: u8 = match range.to.as_deref() {
        Some(&Expr::Lit(ExprLit {
            lit: Lit::Int(ref int),
            ..
        })) => int.base10_parse()?,
        Some(other) => return Err(syn::Error::new_spanned(other, "Invalid range end")),
        None => 255,
    };
    if let ExprRange {
        limits: RangeLimits::HalfOpen(_),
        to: Some(_),
        ..
    } = range
    {
        range_end = range_end
            .checked_sub(1)
            .ok_or_else(|| syn::Error::new_spanned(&range.to, "Invalid range"))?;
    };

    let mut handlers = Vec::new();
    for index in range_start..=range_end {
        let handler = quote! {
            {
                extern "x86-interrupt" fn handler(stack_frame: &mut InterruptStackFrame) {
                    #function(stack_frame, #index.into());
                }
                (#idt)[#index.into()].set_handler_fn(handler);
            }
        };
        handlers.push(handler);
    }

    let ret = quote! {
        #(#handlers)*
    };

    Ok(ret.into())
}
