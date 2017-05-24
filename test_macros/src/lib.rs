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
#![feature(rustc_private, proc_macro)]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate syntax;

extern crate test;
use test::KvmTestMetaData;

use syn::fold;
use syn::parse::IResult;
use std::string;
use proc_macro::TokenStream;
use quote::ToTokens;

/// Add a additional meta-data and setup functions for a KVM based test.
fn generate_kvmtest_meta_data(test_ident: &syn::Ident) -> (syn::Ident, quote::Tokens) {
    // Create some test meta-data
    let test_name = test_ident.as_ref();
    let setup_fn_ident = syn::Ident::new(String::from(test_name) + "_setup");
    let struct_ident = syn::Ident::new(String::from(test_name) + "_kvm_meta_data");

    (struct_ident.clone(),
     quote! {
        extern crate test;
        use self::test::KvmTestMetaData;
        #[link_section = ".kvm"]
        #[used]
        static #struct_ident: KvmTestMetaData = KvmTestMetaData { mbz: 0, meta: "test"  };

        /// The generated impl
        fn #setup_fn_ident() {
            log!("blabla");
        }
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
        IResult::Error => panic!("Unable to generate reference to meta data"),
    };

    test_block.stmts.insert(0, stmt);
}


#[proc_macro_attribute]
pub fn kvmattrs(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = args.to_string();
    let mut input = input.to_string();
    let mut ast = syn::parse_item(&input).unwrap();
    let ident = ast.ident.clone();
    let (meta_data_ident, new_code) = generate_kvmtest_meta_data(&ident);

    {
        match &mut ast.node {
            &mut syn::ItemKind::Fn(_, _, _, _, _, ref mut block) => {
                let mod_test_code = insert_meta_data_reference(&meta_data_ident, block);
                println!("{:#?}", block);
            }
            _ => panic!("Not a function!"),
        };
    }
    let mut token = quote::Tokens::new();
    ast.to_tokens(&mut token);
    token.append(new_code);

    //input += new_code.to_string().as_str();
    token.to_string().parse().unwrap()
}
