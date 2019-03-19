#![recursion_limit = "128"]

extern crate proc_macro;

use core::fmt::Debug;
use std::collections::HashMap;
use std::process::Command;

use proc_macro2::{Span, TokenStream, TokenTree};
use proc_macro2::token_stream::IntoIter;
use quote::{quote, quote_spanned};
use syn::{Attribute, AttributeArgs, Data, Expr, ExprStruct, Fields, FieldValue, FnArg, Ident, Item, ItemFn, Lit, LitStr, Meta, parse2, parse_macro_input, parse_quote, parse_str, Pat, Path, Type};
use syn::parse_quote::parse;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;

/// See the [crate documentation](index.html) for details
#[proc_macro_attribute]
pub fn ivory_export(attr: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let item = syn::parse2::<Item>(input).unwrap();
    let attr = parse_macro_input!(attr as AttributeArgs);

    let output = match item {
        Item::Fn(itemFn) => {
            export_fn(itemFn).into()
        }
        _ => unimplemented!()
    };

    // panic!("{}", output);

    output
}

fn export_fn(item: ItemFn) -> TokenStream {
    let span = item.span();
    let name = item.ident;
    let name_str = name.to_string();
    let meta_name = Ident::new(&format!("FUNCTION_META_{}", name_str.to_uppercase()), span);
    let body = item.block;
    let decl = item.decl;
    if decl.generics.gt_token.is_some() {
        unimplemented!("generics are not supported for exported functions");
    }

    let arg_defs = decl.inputs.into_iter()
        .map(get_arg_info)
        .map(|(name, _type, is_ref)| {
        quote!(::ivory::zend::ArgInfo::new(::ivory::c_str!(#name), false, false, #is_ref))
    });

    quote! {
        #[no_mangle]
        pub extern "C" fn #name(data: &ExecuteData, retval: &Value) {
            let result = #body;
        }

        const #meta_name: ::ivory::zend::FunctionMeta = ::ivory::zend::FunctionMeta{
            name: {concat!(#name_str, "\0").as_ptr() as *const ::libc::c_char},
            func: #name,
            args: &[ #(#arg_defs),*]
        };
    }
}

fn get_arg_info(arg: FnArg) -> (String, Type, bool) {
    match arg {
        FnArg::Captured(cap) => {
            let arg_type = cap.ty;
            match cap.pat {
                Pat::Ident(ident_pat) => {
                    (ident_pat.ident.to_string(), arg_type, ident_pat.by_ref.is_some())
                },
                Pat::Ref(ref_pat) => unimplemented!(),
                _ => panic!()
            }
        },
        _ => panic!("only normal function arguments are supported")
    }
}

/// See the [crate documentation](index.html) for details
#[proc_macro]
pub fn ivory_module(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let span = input.span();

    let mut tokens = input.into_iter();
    let token = tokens.next().unwrap();
    let group = match token {
        TokenTree::Group(group) => {
            group
        }
        _ => panic!("macro input must be a group")
    };

    let fields = group.stream();

    let struct_def = quote! {
        ::ivory::zend::PhpModule {
            #fields
        }
    };
    let function_names = get_function_names(struct_def);
    let funcs = get_funcs(function_names, span);

    let fields = into_c_str(fields);

    let result = quote! {
        const MODULE_INFO: ::ivory::zend::PhpModule = ::ivory::zend::PhpModule {
            #fields
        };

        extern "C" fn php_module_info() {
            ::ivory::info::php_print_module_info(&MODULE_INFO.info);
        }

        #[no_mangle]
        pub extern "C" fn get_module() -> *mut ::ivory::zend::ModuleInternal {
            let mut entry = Box::new(::ivory::zend::ModuleInternal::new(MODULE_INFO.name, MODULE_INFO.version));

            entry.set_info_func(php_module_info);

            let args = vec![
                ::ivory::zend::ArgInfo::new(::ivory::c_str!("name"), false, false, false),
                ::ivory::zend::ArgInfo::new(::ivory::c_str!("foo"), false, false, false),
            ];

            #funcs;

            entry.set_functions(funcs);

            Box::into_raw(entry)
        }
    };

    // panic!("{}", result);

    result.into()
}

fn into_c_str(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().map(|token| {
        match token.clone() {
            TokenTree::Literal(lit) => {
                let mut tokens = TokenStream::new();
                tokens.extend(vec![token.clone()]);
                match syn::parse2::<LitStr>(tokens) {
                    Ok(litStr) => {
                        let val = litStr.value();
                        let tokens = quote! {
                            { concat!(#val, "\0").as_ptr() as *const ::libc::c_char }
                        };
                        if let Some(tree) = tokens.into_iter().next() {
                            tree
                        } else {
                            panic!();
                        }
                    }
                    Err(_) => token
                }
            }
            _ => token
        }
    }).collect();
    let mut output = TokenStream::new();
    output.extend(tokens.into_iter());
    output
}

fn get_function_names(struct_def: TokenStream) -> Vec<String> {
    let expr: Expr = parse2(struct_def).unwrap();
    let expr = get_field_expr(expr, "functions").unwrap();
    match expr {
        Expr::Reference(ref_expr) => {
            match *ref_expr.expr {
                Expr::Array(arr) => {
                    arr.elems.into_iter().map(|element: Expr| {
                        let tokens: TokenStream = quote!(#element);
                        let tree = tokens.into_iter().next().unwrap();
                        match tree {
                            TokenTree::Ident(ident) => {
                                ident.to_string()
                            }
                            _ => panic!()
                        }
                    }).collect()
                }
                _ => panic!()
            }
        }
        _ => panic!()
    }
}

fn get_field_expr(expr: Expr, field_name: &str) -> Option<Expr> {
    let fields: Punctuated<FieldValue, syn::token::Comma> = match expr {
        Expr::Struct(expr) => expr.fields,
        _ => panic!("invalid struct")
    };
    for field in fields {
        if let syn::Member::Named(ident) = &field.member {
            let name = ident.to_string();
            if &name == field_name {
                return Some(field.expr);
            }
        }
    }
    None
}

fn get_funcs(names: Vec<String>, span: Span) -> TokenStream {
    let definitions = names.into_iter().map(|name| {
        let meta_name = Ident::new(&format!("FUNCTION_META_{}", name.to_uppercase()), span);
        quote! {
            #meta_name.as_function()
        }
    });

    quote! {
        let funcs = vec![
            #(#definitions),*
        ];
    }
}