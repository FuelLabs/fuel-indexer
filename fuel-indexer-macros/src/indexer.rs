use crate::native::handler_block_native;
use crate::parse::IndexerConfig;
use crate::schema::process_graphql_schema;
use crate::wasm::handler_block_wasm;
use fuel_indexer_schema::BlockData;
use fuels_core::{
    abi_encoder::ABIEncoder, code_gen::abigen::Abigen, json_abi::ABIParser,
    source::Source,
};
use fuels_types::JsonABI;
use sha2::{Digest, Sha256};
use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, Ident, Item, ItemFn, ItemMod, PatType, Type};

const DISALLOWED: &[&str] = &["Vec"];

fn get_json_abi(abi: String) -> JsonABI {
    let src = match Source::parse(abi) {
        Ok(src) => src,
        Err(e) => {
            proc_macro_error::abort_call_site!(
                "`abi` must be a file path to valid json abi! {:?}",
                e
            )
        }
    };

    let source = match src.get() {
        Ok(s) => s,
        Err(e) => {
            proc_macro_error::abort_call_site!("Could not fetch JSON ABI. {:?}", e)
        }
    };

    match serde_json::from_str(&source) {
        Ok(parsed) => parsed,
        Err(e) => {
            proc_macro_error::abort_call_site!("Invalid JSON from ABI spec! {:?}", e)
        }
    }
}

fn rust_name(ty: &str) -> Ident {
    if ty.contains(' ') {
        let ty = ty
            .split(' ')
            .last()
            .unwrap()
            .to_string()
            .to_ascii_lowercase();
        format_ident! { "{}_decoded", ty }
    } else {
        let ty = ty.to_ascii_lowercase();
        format_ident! { "{}_decoded", ty }
    }
}

fn rust_type(ty: &String) -> Ident {
    if ty.contains(' ') {
        let ty = ty.split(' ').last().unwrap().to_string();
        format_ident! { "{}", ty }
    } else {
        format_ident! { "{}", ty }
    }
}

fn type_id(bytes: &[u8]) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();

    let mut output = [0u8; 8];
    output.copy_from_slice(&result[..8]);

    u64::from_be_bytes(output)
}

fn is_primitive(ty: &Ident) -> bool {
    let ident_str = ty.to_string();
    // TODO: complete the list
    matches!(
        ident_str.as_str(),
        "u8" | "u16" | "u32" | "u64" | "bool" | "BlockData"
    )
}

fn decode_snippet(ty_id: u64, ty: &Ident, name: &Ident) -> proc_macro2::TokenStream {
    if is_primitive(ty) {
        // TODO: do we want decoder for primitive? Might need something a little smarte to identify what the primitive is for... and to which handler it will go.
        quote! {
            #ty_id => {
                Logger::warn("Skipping primitive decoder");
            }
        }
    } else {
        quote! {
            #ty_id => {
                let decoded = ABIDecoder::decode(&#ty::param_types(), &data).expect("Failed decoding");
                let obj = #ty::new_from_tokens(&decoded);
                self.#name.push(obj);
            }
        }
    }
}

fn process_fn_items(
    abi: String,
    input: ItemMod,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let disallowed_types: HashSet<String> =
        HashSet::from_iter(DISALLOWED.iter().map(|s| s.to_string()));

    if input.content.is_none() || input.content.as_ref().unwrap().1.is_empty() {
        proc_macro_error::abort_call_site!(
            "No module body, must specify at least one handler function!"
        )
    }

    let parsed = get_json_abi(abi);

    let mut types = HashSet::new();
    let mut selectors = Vec::new();
    let mut decoders = Vec::new();
    let mut type_vecs = Vec::new();
    let mut dispatchers = Vec::new();

    for function in parsed {
        if function.outputs.len() > 1 {
            proc_macro_error::abort_call_site!("Multiple returns not supported!")
        }

        let sig =
            match ABIParser::new().build_fn_selector(&function.name, &function.inputs) {
                Ok(s) => s,
                Err(e) => {
                    proc_macro_error::abort_call_site!(
                        "Could not calculate fn selector! {:?}",
                        e
                    )
                }
            };

        let selector = ABIEncoder::encode_function_selector(sig.as_bytes());
        let selector = u64::from_be_bytes(selector);

        if let Some(out) = &function.outputs.first() {
            let output = out.type_field.clone();

            let ty = rust_type(&output);
            let name = rust_name(&output);
            let ty_id = type_id(ty.to_string().as_bytes());

            if !types.contains(&ty_id) {
                type_vecs.push(quote! {
                    #name: Vec<#ty>
                });

                decoders.push(decode_snippet(ty_id, &ty, &name));
                types.insert(ty_id);
            }

            selectors.push(quote! {
                #selector => #ty_id,
            });
        }

        for input in function.inputs {
            let input = input.type_field.clone();

            let ty = rust_type(&input);
            let name = rust_name(&input);
            let ty_id = type_id(ty.to_string().as_bytes());

            if !types.contains(&ty_id) {
                type_vecs.push(quote! {
                    #name: Vec<#ty>
                });

                decoders.push(decode_snippet(ty_id, &ty, &name));
                types.insert(ty_id);
            }
        }
    }

    let contents = input.content.unwrap().1;
    let mut handler_fns = Vec::with_capacity(contents.len());

    fn is_block_fn(input: &ItemFn) -> bool {
        if input.attrs.len() == 1 {
            let path = &input.attrs[0].path;
            if path.get_ident().unwrap() == "block" {
                return true;
            }
        }
        false
    }

    let mut block_dispatchers = Vec::new();

    let mut blockdata_decoding = quote! {};

    for item in contents {
        match item {
            Item::Fn(fn_item) => {
                let mut input_checks = Vec::new();
                let mut arg_list = Vec::new();

                if is_block_fn(&fn_item) {
                    let input = BlockData::ident();

                    let ty = rust_type(&input);
                    let name = rust_name(&input);
                    let ty_id = type_id(ty.to_string().as_bytes());

                    if !types.contains(&ty_id) {
                        type_vecs.push(quote! {
                            #name: Vec<#ty>
                        });

                        types.insert(ty_id);
                    }

                    block_dispatchers.push(quote! { self.#name.push(data); });

                    blockdata_decoding =
                        quote! { decoder.decode_blockdata(block.clone()); };
                }

                for inp in &fn_item.sig.inputs {
                    match inp {
                        FnArg::Receiver(_) => {
                            proc_macro_error::abort_call_site!(
                                "`self` argument not allowed in handler function."
                            )
                        }
                        FnArg::Typed(PatType { ty, .. }) => {
                            if let Type::Path(path) = &**ty {
                                let path = path.path.segments.last().unwrap();
                                let ty_id = type_id(path.ident.to_string().as_bytes());

                                if disallowed_types.contains(&path.ident.to_string()) {
                                    proc_macro_error::abort_call_site!(
                                        "Type {:?} currently not supported",
                                        path.ident
                                    )
                                }

                                if !types.contains(&ty_id) {
                                    proc_macro_error::abort_call_site!(
                                        "Type {:?} not defined in the ABI.",
                                        path.ident,
                                    )
                                }

                                let name = rust_name(&path.ident.to_string());
                                input_checks.push(quote! { self.#name.len() > 0 });

                                arg_list.push(quote! { self.#name[0].clone() });
                            } else {
                                proc_macro_error::abort_call_site!(
                                    "Arguments must be types defined in the abi.json."
                                )
                            }
                        }
                    }
                }

                let fn_name = &fn_item.sig.ident;

                dispatchers.push(quote! {
                    if ( #(#input_checks)&&* ) {
                        #fn_name(#(#arg_list),*);
                    }
                });

                handler_fns.push(fn_item);
            }
            i => {
                proc_macro_error::abort_call_site!(
                    "Unsupported item in indexer module {:?}",
                    i
                )
            }
        }
    }

    let decoder_struct = quote! {
        #[derive(Default)]
        struct Decoders {
            #(#type_vecs),*
        }

        impl Decoders {
            fn selector_to_type_id(&self, sel: u64) -> u64 {
                match sel {
                    #(#selectors)*
                    //TODO: should handle this a little more gently
                    _ => panic!("Unknown type id!"),
                }
            }

            fn decode_type(&mut self, ty_id: u64, data: Vec<u8>) {
                match ty_id {
                    #(#decoders),*
                    _ => panic!("Unkown type id {}", ty_id),
                }
            }

            pub fn decode_blockdata(&mut self, data: BlockData) {
                #(#block_dispatchers)*
            }

            pub fn decode_return_type(&mut self, sel: u64, data: Vec<u8>) {
                let ty_id = self.selector_to_type_id(sel);

                self.decode_type(ty_id, data);
            }

            pub fn decode_log_data(&self) {
                todo!("Finish this off")
            }

            pub fn dispatch(&self) {
                #(#dispatchers)*
            }
        }
    };
    (
        quote! {
            for block in blocks {
                let mut decoder = Decoders::default();

                #blockdata_decoding

                for tx in block.transactions {
                    let mut return_types = Vec::new();

                    for receipt in tx {
                        match receipt {
                            Receipt::Call { param1, ..} => {
                                return_types.push(param1);
                            }
                            Receipt::ReturnData { data, .. } => {
                                let selector = return_types.pop().expect("No return type available!");
                                decoder.decode_return_type(selector, data);
                            }
                            other => {
                                Logger::info("This type is not handled yet!");
                            }
                        }
                    }

                    decoder.dispatch();
                }
                // TODO: save block height process to DB...
            }
        },
        quote! {
            #decoder_struct

            #(#handler_fns)*
        },
    )
}

pub fn process_indexer_module(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let config = parse_macro_input!(attrs as IndexerConfig);

    let IndexerConfig {
        abi,
        namespace,
        schema,
        native,
        ..
    } = config;

    let indexer = parse_macro_input!(item as ItemMod);

    let (abi_string, schema_string) = match std::env::var("COMPILE_TEST_PREFIX") {
        Ok(prefix) => {
            let prefixed = std::path::Path::new(&prefix).join(&abi);
            let abi_string = prefixed.into_os_string().to_str().unwrap().to_string();
            let prefixed = std::path::Path::new(&prefix).join(&schema);
            let schema_string = prefixed.into_os_string().to_str().unwrap().to_string();
            (abi_string, schema_string)
        }
        Err(_) => (abi, schema),
    };

    let abi_tokens = match Abigen::new(&namespace, &abi_string) {
        Ok(abi) => match abi.no_std().expand() {
            Ok(tokens) => tokens,
            Err(e) => {
                proc_macro_error::abort_call_site!(
                    "Could not generate tokens for abi! {:?}",
                    e
                )
            }
        },
        Err(e) => {
            proc_macro_error::abort_call_site!("Could not generate abi object! {:?}", e)
        }
    };

    let graphql_tokens = process_graphql_schema(namespace, schema_string);

    let (handler_block, fn_items) = process_fn_items(abi_string, indexer);

    let handler_block = if native {
        handler_block_native(handler_block)
    } else {
        handler_block_wasm(handler_block)
    };

    let output = quote! {
        use alloc::{format, vec, vec::Vec};
        use fuel_indexer_plugin::{Entity, Logger};
        use fuel_indexer_plugin::types::*;
        use fuels_core::{abi_decoder::ABIDecoder, ParamType, Parameterize};
        use fuel_tx::Receipt;
        use fuel_indexer_macros::block;

        #abi_tokens

        #graphql_tokens

        #handler_block

        #fn_items
    };

    proc_macro::TokenStream::from(output)
}

pub fn process_block_attribute_fn(_attrs: TokenStream, item: TokenStream) -> TokenStream {
    let block_fn = parse_macro_input!(item as ItemFn);

    let output = quote! {

        #block_fn
    };

    proc_macro::TokenStream::from(output)
}
