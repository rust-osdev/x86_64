#![feature(rustc_private)]
#![feature(proc_macro)]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate syntax;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn panics_note(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = args.to_string();
    let mut input = input.to_string();

    assert!(args.starts_with("= \""),
            "`#[panics_note]` requires an argument of the form `#[panics_note = \"panic note \
             here\"]`");

    // Get just the bare note string
    let panics_note = args.trim_matches(&['=', ' ', '"'][..]);

    // The input will include all docstrings regardless of where the attribute is placed,
    // so we need to find the last index before the start of the item
    let insert_idx = idx_after_last_docstring(&input);

    // And insert our `### Panics` note there so it always appears at the end of an item's docs
    input.insert_str(insert_idx, &format!("/// # Panics \n/// {}\n", panics_note));

    input.parse().unwrap()
}

// `proc-macro` crates can contain any kind of private item still
fn idx_after_last_docstring(input: &str) -> usize {
    // Skip docstring lines to find the start of the item proper
    input.lines().skip_while(|line| line.trim_left().starts_with("///")).next()
        // Find the index of the first non-docstring line in the input
        // Note: assumes this exact line is unique in the input
        .and_then(|line_after| input.find(line_after))
        // No docstrings in the input
        .unwrap_or(0)
}


fn generate_kvm_setup(ident: syn::Ident) -> quote::Tokens {
    quote! {
        // The generated impl
        fn #ident() {
            log!("blabla");
        }
    }
}

use std::string;

#[proc_macro_attribute]
pub fn kvmattrs(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = args.to_string();
    let mut input = input.to_string();

    let ast = syn::parse_item(&input).unwrap();
    let new_fn_ident = syn::Ident::new(String::from(ast.ident.as_ref()) + "_setup");
    println!("{:?}", new_fn_ident);

    // Get just the bare note string
    let panics_note = args.trim_matches(&['=', ' ', '"'][..]);
    let new_code: TokenStream = generate_kvm_setup(new_fn_ident).parse().unwrap();

    input += new_code.to_string().as_str();
    input.parse().unwrap()
}
