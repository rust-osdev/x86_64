use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use std::{fmt::Display, ops::RangeInclusive};
use syn::{
    parse::Parser, punctuated::Punctuated, spanned::Spanned, token::Comma, Expr, ExprLit, ExprPath,
    ExprRange, ExprReference, Lit, RangeLimits,
};

extern crate proc_macro;

const RESERVED_HANDLERS: &[u8] = &[15, 21, 22, 23, 24, 25, 26, 27, 28, 29, 31];
const HANDLERS_WITH_ERR_CODE: &[u8] = &[8, 10, 11, 12, 13, 14, 17, 30];

#[proc_macro]
pub fn set_general_handler(item: TokenStream) -> TokenStream {
    set_general_handler_impl(item).unwrap_or_else(|err| err.to_compile_error().into())
}

fn set_general_handler_impl(item: TokenStream) -> syn::Result<TokenStream> {
    let input = Punctuated::<Expr, Comma>::parse_separated_nonempty.parse(item)?;
    let (idt, function, range) = parse_input(&input)?;

    let mut handlers = Vec::new();
    for index in range {
        if RESERVED_HANDLERS.contains(&index) {
            continue; // skip reserved handlers
        }
        let function_call = if HANDLERS_WITH_ERR_CODE.contains(&index) {
            quote_spanned!(function.span() => {
                #function(stack_frame, #index.into(), Some(error_code));
            })
        } else {
            quote_spanned!(function.span() => {
                #function(stack_frame, #index.into(), None);
            })
        };

        let stack_frame_arg =
            quote! { stack_frame: &mut x86_64::structures::idt::InterruptStackFrame };

        let handler_with_err_code = quote! {
            extern "x86-interrupt" fn handler(#stack_frame_arg, error_code: u64) {
                #function_call
            }
        };

        let set_handler_fn = match index {
            8 => quote! {
                extern "x86-interrupt" fn handler(#stack_frame_arg, error_code: u64) -> ! {
                    #function_call
                    panic!("General handler returned on double fault");
                }
                (#idt).double_fault.set_handler_fn(handler);
            },
            10 => quote! {
                #handler_with_err_code
                (#idt).invalid_tss.set_handler_fn(handler);
            },
            11 => quote! {
                #handler_with_err_code
                (#idt).segment_not_present.set_handler_fn(handler);
            },
            12 => quote! {
                #handler_with_err_code
                (#idt).stack_segment_fault.set_handler_fn(handler);
            },
            13 => quote! {
                #handler_with_err_code
                (#idt).general_protection_fault.set_handler_fn(handler);
            },
            14 => quote! {
                extern "x86-interrupt" fn handler(#stack_frame_arg, error_code: x86_64::structures::idt::PageFaultErrorCode) {
                    let error_code = error_code.bits();
                    #function_call
                }
                (#idt).page_fault.set_handler_fn(handler);
            },
            17 => quote! {
                #handler_with_err_code
                (#idt).alignment_check.set_handler_fn(handler);
            },
            18 => quote! {
                extern "x86-interrupt" fn handler(#stack_frame_arg) -> ! {
                    #function_call
                    panic!("General handler returned on machine check exception");
                }
                (#idt).machine_check.set_handler_fn(handler);
            },
            30 => quote! {
                #handler_with_err_code
                (#idt).security_exception.set_handler_fn(handler);
            },
            index => quote! {
                extern "x86-interrupt" fn handler(#stack_frame_arg) {
                    #function_call
                }
                (#idt)[#index.into()].set_handler_fn(handler);
            },
        };

        // double `{{` to create new scope
        handlers.push(quote! {{
            #set_handler_fn
        }});
    }

    let ret = quote! {
        #(#handlers)*
    };

    // uncomment to print generated code:
    // println!("{}", ret);

    Ok(ret.into())
}

fn parse_input(
    input: &Punctuated<Expr, Comma>,
) -> syn::Result<(&ExprReference, &ExprPath, RangeInclusive<u8>)> {
    if input.len() < 2 || input.len() > 3 {
        return Err(err(input, "expected 2 or 3 arguments"));
    }

    let idt = match &input[0] {
        Expr::Reference(r) if r.mutability.is_some() => r,
        other => return Err(err(other, "expected a `&mut` reference to an IDT")),
    };

    let function = match &input[1] {
        Expr::Path(path) => path,
        other => return Err(err(other, "expected the name of a handler function")),
    };

    let range = if input.len() == 3 {
        match &input[2] {
            Expr::Lit(lit) => match &lit.lit {
                Lit::Int(int) => {
                    let index: u8 = int.base10_parse()?;
                    index..=index
                }
                other => return Err(err(other, "expected index or range")),
            },
            Expr::Range(range) => parse_range(&range)?,
            other => return Err(err(other, "expected index or range")),
        }
    } else {
        0..=255
    };

    Ok((idt, function, range))
}

fn parse_range(range: &ExprRange) -> syn::Result<RangeInclusive<u8>> {
    let range_start: u8 = match range.from.as_deref() {
        Some(&Expr::Lit(ExprLit {
            lit: Lit::Int(ref int),
            ..
        })) => int.base10_parse()?,
        Some(other) => return Err(err(other, "Invalid range start")),
        None => 0,
    };
    let mut range_end: u8 = match range.to.as_deref() {
        Some(&Expr::Lit(ExprLit {
            lit: Lit::Int(ref int),
            ..
        })) => int.base10_parse()?,
        Some(other) => return Err(err(other, "Invalid range end")),
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
            .ok_or_else(|| err(&range.to, "Invalid range"))?;
    };

    Ok(range_start..=range_end)
}

fn err(tokens: impl quote::ToTokens, message: impl Display) -> syn::Error {
    syn::Error::new_spanned(tokens, message)
}
