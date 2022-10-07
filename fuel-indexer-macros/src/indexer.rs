use crate::native::handler_block_native;
use crate::parse::IndexerConfig;
use crate::schema::process_graphql_schema;
use crate::wasm::handler_block_wasm;
use fuel_indexer_schema::{
    type_id,
    types::{
        BlockData, Identity, Log, LogData, ReceiptType, Transfer, FUEL_TYPES_NAMESPACE,
    },
};
use fuels_core::{
    code_gen::{abigen::Abigen, function_selector::resolve_fn_selector},
    source::Source,
    utils::first_four_bytes_of_sha256_hash,
};
use fuels_types::{ProgramABI, TypeDeclaration};
use std::collections::{HashMap, HashSet};

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, Ident, Item, ItemFn, ItemMod, PatType, Type};

const DISALLOWED: &[&str] = &["Vec"];
const EMPTY_TUPLE_TYPE: &str = "()";

lazy_static! {
    static ref FUEL_PRIMITIVES: HashSet<&'static str> = HashSet::from([
        Transfer::path_ident_str(),
        BlockData::path_ident_str(),
        "B256",
        Log::path_ident_str(),
        LogData::path_ident_str()
    ]);
}

fn get_json_abi(abi: String) -> ProgramABI {
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

fn rust_name_str(ty: &str) -> Ident {
    format_ident! { "{}_decoded", ty.to_ascii_lowercase() }
}

fn rust_name(ty: &TypeDeclaration) -> Ident {
    if ty.components.is_some() {
        let ty = ty.type_field.split(' ').last().unwrap().to_string();
        rust_name_str(&ty)
    } else {
        let ty = ty.type_field.replace(['[', ']'], "_");
        rust_name_str(&ty)
    }
}

fn rust_type(ty: &TypeDeclaration) -> proc_macro2::TokenStream {
    if ty.components.is_some() {
        let ty = ty.type_field.split(' ').last().unwrap().to_string();
        let ident = format_ident! { "{}", ty };
        quote! { #ident }
    } else {
        // TODO: decode all the types
        match ty.type_field.as_str() {
            "bool" => quote! { bool },
            "u8" => quote! { u8 },
            "u16" => quote! { u16 },
            "u32" => quote! { u32 },
            "u64" => quote! { u64 },
            "b256" => quote! { B256 },
            o if o.starts_with("str[") => quote! { String },
            o => {
                proc_macro_error::abort_call_site!("Unrecognized primitive type! {:?}", o)
            }
        }
    }
}

fn is_fuel_primitive(ty: &proc_macro2::TokenStream) -> bool {
    // TODO: complete as needed
    let ident_str = ty.to_string();
    FUEL_PRIMITIVES.contains(ident_str.as_str())
}

fn is_rust_primitive(ty: &proc_macro2::TokenStream) -> bool {
    let ident_str = ty.to_string();
    // TODO: complete the list
    matches!(
        ident_str.as_str(),
        "u8" | "u16" | "u32" | "u64" | "bool" | "String"
    )
}

fn is_primitive(ty: &proc_macro2::TokenStream) -> bool {
    is_rust_primitive(ty) || is_fuel_primitive(ty)
}

fn decode_snippet(
    ty_id: usize,
    ty: &proc_macro2::TokenStream,
    name: &Ident,
) -> proc_macro2::TokenStream {
    if is_primitive(ty) {
        // TODO: do we want decoder for primitive? Might need something a little smarter to
        // identify what the primitive is for and to which handler it will go.
        quote! {
            #ty_id => {
                Logger::warn("Skipping primitive decoder");
            }
        }
    } else {
        quote! {
            #ty_id => {
                let decoded = ABIDecoder::decode_single(&#ty::param_type(), &data).expect("Failed decoding");
                let obj = #ty::from_token(decoded).expect("Failed detokenizing");
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
            "No module body, must specify at least one handler function."
        )
    }

    let parsed = get_json_abi(abi);

    let mut types = HashSet::new();
    let mut selectors = Vec::new();
    let mut decoders = Vec::new();
    let mut type_vecs = Vec::new();
    let mut dispatchers = Vec::new();

    let mut type_map = HashMap::new();
    let mut type_ids = FUEL_PRIMITIVES
        .iter()
        .map(|x| (x.to_string(), type_id(FUEL_TYPES_NAMESPACE, x) as usize))
        .collect::<HashMap<String, usize>>();

    for typ in parsed.types {
        if typ.type_field == EMPTY_TUPLE_TYPE {
            println!("Won't process empty tuple type.");
            continue;
        }

        type_map.insert(typ.type_id, typ.clone());

        let ty = rust_type(&typ);
        let name = rust_name(&typ);
        let ty_id = typ.type_id;

        type_ids.insert(ty.to_string(), ty_id);

        if !types.contains(&ty_id) {
            type_vecs.push(quote! {
                #name: Vec<#ty>
            });

            decoders.push(decode_snippet(ty_id, &ty, &name));
            types.insert(ty_id);
        }
    }

    for function in parsed.functions {
        let sig = resolve_fn_selector(&function, &type_map);

        let selector = first_four_bytes_of_sha256_hash(&sig);
        let selector = u64::from_be_bytes(selector);
        let ty_id = function.output.type_id;

        selectors.push(quote! {
            #selector => #ty_id,
        });
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
    let mut transfer_dispatchers = Vec::new();
    let mut log_dispatchers = Vec::new();
    let mut logdata_dispatchers = Vec::new();

    let mut blockdata_decoding = quote! {};

    for item in contents {
        match item {
            Item::Fn(fn_item) => {
                let mut input_checks = Vec::new();
                let mut arg_list = Vec::new();

                if is_block_fn(&fn_item) {
                    let path_ident_str = String::from(BlockData::path_ident_str());
                    let type_id = type_id(FUEL_TYPES_NAMESPACE, &path_ident_str) as usize;

                    let typ = TypeDeclaration {
                        type_id,
                        type_field: path_ident_str,
                        type_parameters: None,
                        components: None,
                    };

                    let ty = quote! { BlockData };
                    let name = rust_name(&typ);
                    let ty_id = typ.type_id;

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
                                let path_ident_str = path.ident.to_string();
                                // NOTE: may need to get something else for primitives...
                                let ty_id = match type_ids.get(&path_ident_str) {
                                    Some(id) => id,
                                    None => {
                                        proc_macro_error::abort_call_site!(
                                            "Type with ident {:?} not defined in the ABI.",
                                            path.ident
                                        );
                                    }
                                };

                                if disallowed_types.contains(&path_ident_str) {
                                    proc_macro_error::abort_call_site!(
                                        "Type {:?} currently not supported",
                                        path.ident
                                    )
                                }

                                if !types.contains(ty_id) {
                                    // If the fn parameter type is in `type_ids` but not in `types` that means it's a fuel primitive
                                    // for which there is no ABI JSON

                                    if FUEL_PRIMITIVES.contains(path_ident_str.as_str()) {
                                        let typ = TypeDeclaration {
                                            type_id: *ty_id,
                                            type_field: path_ident_str.clone(),
                                            type_parameters: None,
                                            components: None,
                                        };

                                        let name = rust_name(&typ);

                                        type_vecs.push(quote! {
                                            #name: Vec<#ty>
                                        });

                                        match path_ident_str.into() {
                                            ReceiptType::Transfer => {
                                                transfer_dispatchers.push(quote! { self.#name.push(data); });
                                            }
                                            ReceiptType::Log => {
                                                log_dispatchers.push(quote!{ self.#name.push(data); });
                                            }

                                            ReceiptType::LogData => {
                                                logdata_dispatchers.push(quote!{ self.#name.push(data); });
                                                decoders.push(decode_snippet(*ty_id, &rust_type(&typ), &name));
                                            }
                                            _ => panic!("Unsupported ReceiptType in function signature"),
                                        }
                                    } else {
                                        proc_macro_error::abort_call_site!(
                                            "Type {:?} not defined in the ABI.",
                                            path.ident,
                                        )
                                    }
                                }

                                let name = rust_name_str(&path.ident.to_string());
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
            fn selector_to_type_id(&self, sel: u64) -> usize {
                match sel {
                    #(#selectors)*
                    //TODO: should handle this a little more gently
                    _ => panic!("Unknown type id."),
                }
            }

            pub fn decode_type(&mut self, ty_id: usize, data: Vec<u8>) {
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
                self.decode_type(ty_id, data)
            }

            pub fn decode_transfer(&mut self, data: Transfer) {
                #(#transfer_dispatchers)*
            }

            pub fn decode_log(&mut self, data: Log) {
                #(#log_dispatchers)*
            }

            pub fn decode_logdata(&mut self, rb: u64, data: Vec<u8>) {
                // TODO: Use rb here to determine what the `type` is, then use `type` (the type id) to decode_type()
                self.decode_type(rb as usize, data)
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
                                let selector = return_types.pop().expect("No return type available. <('-'<)");
                                decoder.decode_return_type(selector, data);
                            }

                            Receipt::Transfer { id, to, asset_id, amount, pc, is, .. } => {
                                let data = Transfer{ contract_id: id, to, asset_id, amount, pc, is };
                                decoder.decode_transfer(data);
                            }

                            Receipt::Log { id, ra, rb, .. } => {
                                let data = Log{ contract_id: id, ra, rb };
                                decoder.decode_log(data);
                            }

                            Receipt::LogData { rb, data, ptr, len, id, .. } => {
                                // TODO: use rb to determine which struct from ABI JSON to decode into
                                decoder.decode_logdata(rb, data);

                            }

                            _ => {
                                Logger::info("This type is not handled yet. (>'.')>");
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
                    "Could not generate tokens for abi. {:?}",
                    e
                )
            }
        },
        Err(e) => {
            proc_macro_error::abort_call_site!("Could not generate abi object. {:?}", e)
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
        use fuel_indexer_schema::{serialize, deserialize};
        use fuel_indexer_plugin::{Entity, Logger, types::*};
        use fuels_core::{abi_decoder::ABIDecoder, Parameterize, StringToken, Tokenizable};
        use fuel_tx::Receipt;

        type B256 = [u8; 32];

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
