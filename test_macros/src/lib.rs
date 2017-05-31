#![feature(rustc_private, proc_macro)]
///! This implements the kvmargs macro to customize the execution of KVM based tests.
///!
///! One problem we have to solve here is that we need to store additional data
///! to customize the setup of the virtual CPU in the host, whereas the TokenStream
///! of our macro only allows us to modify the test function itself that already runs inside
///! the virtual machine (see also `test_harness/src/lib.rs`).
///!
///! Using procedural macros, we do the following:
///!   (a) Generate a struct to store additional meta-data for a test
///!   (b) Initialize that struct with values from kvmargs attributes
///!   (b) Store the meta-data in a special ELF section (.kvm) so we can find it later
///!   (c) Make sure the linker won't throw away the meta-data struct by inserting a dummy reference
///!       to it in the test function itself.
///!
///! Obviously, this is a bit of a mess right now, my hope is that such things become
///! easier with better custom test harness support.
///!
extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate syntax;

extern crate test;
use test::KvmTestMetaData;

use syn::*;
use syn::parse::IResult;

use std::string;
use proc_macro::TokenStream;
use quote::ToTokens;
use std::collections::HashMap;

fn parse_kvmtest_args(args: &syn::DeriveInput) -> ((u64, u64), (u16, u32), bool) {
    if args.ident.as_ref() != "Dummy" {
        panic!("Get rid of this hack!");
    }

    // If syn ever implements visitor for MetaItems, this can probably be written simpler:
    let mut identity_map: bool = false;
    let mut physical_memory: Vec<u64> = Vec::with_capacity(2);
    let mut ioport_reads: Vec<u32> = vec![0,0];

    for attr in &args.attrs {
        match attr.value {
            syn::MetaItem::List(ref name, ref kvmattrs) => {
                if name.as_ref() != "kvmattrs" {
                    panic!("Only kvmattr supported at the moment!");
                }

                for kvmattr in kvmattrs {
                    match kvmattr {
                        &syn::NestedMetaItem::MetaItem(ref item) => {
                            match item {
                                &syn::MetaItem::List(ref name, ref innerattrs) => {
                                    match name.as_ref() {
                                        "ram" => {
                                            for (idx, innerattr) in innerattrs.iter().enumerate() {
                                                match innerattr {
                                                    &syn::NestedMetaItem::Literal(syn::Lit::Int(n, _)) =>  {
                                                        physical_memory.push(n);
                                                    },
                                                    _ => panic!("Type mismatch in ram() arguments.")
                                                }
                                            }
                                        }
                                        "ioport" => {
                                            ioport_reads.clear();
                                            for (idx, innerattr) in innerattrs.iter().enumerate() {
                                                match innerattr {
                                                    &syn::NestedMetaItem::Literal(syn::Lit::Int(n, _)) =>  {
                                                        ioport_reads.push(n as u32);
                                                    },
                                                    _ => panic!("Type mismatch in ioport() arguments.")
                                                }
                                            }
                                        }
                                        _ => { panic!("kvmattrs: doesn't support list attribute '{}'", name.as_ref()); }
                                    }
                                },
                                &syn::MetaItem::Word(ref name) => {
                                    match name.as_ref() {
                                        "identity_map" => identity_map = true,
                                        _ => panic!("kvmattrs: doesn't support '{}'", name.as_ref())
                                    }
                                }
                                &syn::MetaItem::NameValue(ref name, ref lit) => println!("{:?}", name.as_ref()),
                                _ => panic!("kvmattrs: can't handle NameValue attribute...")
                            }
                        },
                        _ => { panic!("Can't handle this..."); }
                    }
                }
            }
            _ => { panic!("Only list supported at the moment!"); }
        }
    }

    if physical_memory.len() != 2 {
        panic!("ram() takes two values (x,y)");
    }
    if ioport_reads.len() != 2 {
        panic!("in() takes two values (x,y)");
    }

    ((physical_memory[0], physical_memory[1]), (ioport_reads[0] as u16, ioport_reads[1]), identity_map)
}

/// Add a additional meta-data and setup functions for a KVM based test.
fn generate_kvmtest_meta_data(test_ident: &syn::Ident, args: &syn::DeriveInput) -> (syn::Ident, quote::Tokens) {
    // Create some test meta-data
    let test_name = test_ident.as_ref();
    let setup_fn_ident = syn::Ident::new(String::from(test_name) + "_setup");
    let struct_ident = syn::Ident::new(String::from(test_name) + "_kvm_meta_data");

    let (physical_memory, ioport_reads, identity_map) = parse_kvmtest_args(args);

    (struct_ident.clone(),
     quote! {
        #[link_section = "kvm"]
        #[allow(non_upper_case_globals)]
        static #struct_ident: KvmTestMetaData = KvmTestMetaData {
            mbz: 0,
            meta: #test_name,
            identity_map: #identity_map,
            physical_memory: #physical_memory,
            ioport_reads: #ioport_reads
        };
    })

}

/// Inserts a reference to the corresponding KvmTestData struct of a test function
/// i.e., the test function fn test { foo(); } is changed to
/// fn test() { assert!(foo_kvm_meta_data.mbz == 0); foo(); }
///
/// This makes sure that the linker won't throw away the symbol to the meta data struct.
/// An alternative would be to make sure test code is linked with --whole-archive,
/// but I'm not sure how to do that in rust...
fn insert_meta_data_reference(struct_ident: &syn::Ident, test_block: &mut syn::Block) {
    let stmt_string = format!("assert!({}.mbz == 0);", struct_ident);
    let stmt = match syn::parse::stmt(stmt_string.as_str()) {
        IResult::Done(stmt_str, stmt) => stmt,
        IResult::Error => panic!("Unable to generate reference to meta-data"),
    };
    test_block.stmts.insert(0, stmt);
}

/// Inserts an IO out (outw) instruction at the end of the test function
/// that writes to port 0xf4 with value 0x0. outw will cause a vmexit.
/// This particular port, payload combination is handled as a special case
/// in the test runner to signal that the test has completed.
fn insert_test_shutdown(struct_ident: &syn::Ident, test_block: &mut syn::Block) {
    let stmt_exit_test = String::from("unsafe { x86::shared::io::outw(0xf4, 0x00); }");
    let stmt = match syn::parse::stmt(stmt_exit_test.as_str()) {
        IResult::Done(stmt_str, stmt) => stmt,
        IResult::Error => panic!("Unable to generate test exit instruction."),
    };
    test_block.stmts.push(stmt);
}


#[proc_macro_attribute]
pub fn kvmattrs(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = args.to_string();
    let input = input.to_string();

    let mut ast = syn::parse_item(&input).unwrap();

    // FIXME, see also https://github.com/dtolnay/syn/issues/86
    let derive_input_hack = format!("#[kvmattrs{}] struct Dummy;", args);
    let args_ast = syn::parse_derive_input(&derive_input_hack).unwrap();
    println!("1");

    // Generate meta-data struct
    let ident = ast.ident.clone();
    let (meta_data_ident, new_code) = generate_kvmtest_meta_data(&ident, &args_ast);

    // Insert reference to meta-data in test
    match &mut ast.node {
        &mut syn::ItemKind::Fn(_, _, _, _, _, ref mut block) => {
            insert_meta_data_reference(&meta_data_ident, block);
            insert_test_shutdown(&meta_data_ident, block);
        }
        _ => panic!("Not a function!"),
    };

    // Merge everything together:
    let mut token = quote::Tokens::new();
    ast.to_tokens(&mut token);
    token.append(new_code);

    // Output this as replacement code for the test function
    token.to_string().parse().unwrap()
}
