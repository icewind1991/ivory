#![recursion_limit = "128"]

extern crate proc_macro;

mod cache;

use proc_macro2::{Span, TokenStream, TokenTree};
use quote::quote;
use syn::spanned::Spanned;
use syn::{parse_macro_input, AttributeArgs, FnArg, Ident, Item, ItemFn, LitStr, Pat, Type};

/// See the [crate documentation](index.html) for details
#[proc_macro_attribute]
pub fn ivory_export(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input: TokenStream = input.into();
    let item = syn::parse2::<Item>(input).unwrap();
    let _attr = parse_macro_input!(attr as AttributeArgs);

    let output = match item {
        Item::Fn(item_fn) => export_fn(item_fn).into(),
        _ => unimplemented!(),
    };

    // panic!("{}", output);

    output
}

fn export_fn(item: ItemFn) -> TokenStream {
    let span = item.span();
    let name = item.ident;
    let name_str = name.to_string();
    cache::cache_function(name_str.clone());
    let meta_name = Ident::new(&format!("FUNCTION_META_{}", name_str.to_uppercase()), span);
    let body = item.block;
    let decl = item.decl;
    if decl.generics.gt_token.is_some() {
        unimplemented!("generics are not supported for exported functions");
    }

    let args: Vec<(String, Type, bool, Span)> = decl.inputs.into_iter().map(get_arg_info).collect();
    let arg_count = args.len() as u32;

    let arg_defs = args.iter().map(|(name, _type, is_ref, _)| {
        quote!(::ivory::zend::ArgInfo::new(::ivory::c_str!(#name), false, false, #is_ref))
    });

    let arg_cast = args.iter().enumerate().map(|(_index, (name, ty, _is_ref, span))| {
        let arg_ident = Ident::new(name, span.clone());
        quote!(
            let #arg_ident: #ty = {
                let result: Result<#ty, ::ivory::CastError> = args.next().unwrap().into();
                match result {
                    Ok(val) => val,
                    Err(err) => {
                        ::ivory::externs::error(::ivory::externs::ErrorLevel::Error, format!("{}", err));
                        return;
                    }
                }
            };
        )
    });

    quote! {
        #[no_mangle]
        pub extern "C" fn #name(data: *const ::ivory::zend::ExecuteData, retval: *mut ::ivory::zend::ZVal) {
            let data: &::ivory::zend::ExecuteData = unsafe { data.as_ref() }.unwrap();
            // the less than case is handled during argument casting
            // this is needed for optional arguments
            if data.num_args() > #arg_count {
                ::ivory::externs::error(::ivory::externs::ErrorLevel::Error, format!("unexpected number of arguments, expected {}, got {}", #arg_count, data.num_args()));
                return;
            }
            let mut args = data.args();
            #(#arg_cast);*
            let result = #body;
        }

        const #meta_name: ::ivory::zend::FunctionMeta = ::ivory::zend::FunctionMeta{
            name: {concat!(#name_str, "\0").as_ptr() as *const ::std::os::raw::c_char},
            func: #name,
            args: &[ #(#arg_defs),*]
        };
    }
}

fn get_arg_info(arg: FnArg) -> (String, Type, bool, Span) {
    match arg {
        FnArg::Captured(cap) => {
            let arg_type = cap.ty;
            match cap.pat {
                Pat::Ident(ident_pat) => (
                    ident_pat.ident.to_string(),
                    arg_type,
                    ident_pat.by_ref.is_some(),
                    ident_pat.span(),
                ),
                Pat::Ref(_ref_pat) => unimplemented!(),
                _ => panic!(),
            }
        }
        _ => panic!("only normal function arguments are supported"),
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
        TokenTree::Group(group) => group,
        _ => panic!("macro input must be a group"),
    };

    let fields = group.stream();

    let funcs = get_funcs(cache::get_functions(), span);

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
    let tokens: Vec<TokenTree> = input
        .into_iter()
        .map(|token| match token.clone() {
            TokenTree::Literal(_lit) => {
                let mut tokens = TokenStream::new();
                tokens.extend(vec![token.clone()]);
                match syn::parse2::<LitStr>(tokens) {
                    Ok(lit_str) => {
                        let val = lit_str.value();
                        let tokens = quote! {
                            { concat!(#val, "\0").as_ptr() as *const ::std::os::raw::c_char }
                        };
                        if let Some(tree) = tokens.into_iter().next() {
                            tree
                        } else {
                            panic!();
                        }
                    }
                    Err(_) => token,
                }
            }
            _ => token,
        })
        .collect();
    let mut output = TokenStream::new();
    output.extend(tokens.into_iter());
    output
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
