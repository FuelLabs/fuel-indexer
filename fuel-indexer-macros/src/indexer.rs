use crate::native::handler_block_native;
use crate::parse::IndexerConfig;
use crate::schema::process_graphql_schema;
use crate::wasm::handler_block_wasm;
use fuel_indexer_lib::{manifest::Manifest, utils::local_repository_root};
use fuel_indexer_schema::{
    type_id,
    types::{
        BlockData, Log, LogData, MessageOut, NativeFuelTypeIdent, ScriptResult, Transfer,
        TransferOut, B256, FUEL_TYPES_NAMESPACE,
    },
};
use fuels_core::{
    code_gen::{abigen::Abigen, function_selector::resolve_fn_selector},
    source::Source,
};
use fuels_types::{param_types::ParamType, ProgramABI, TypeDeclaration};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use syn::{parse_macro_input, FnArg, Ident, Item, ItemMod, PatType, Type};

lazy_static! {
    static ref FUEL_PRIMITIVES: HashSet<&'static str> = HashSet::from([
        Transfer::path_ident_str(),
        BlockData::path_ident_str(),
        B256::path_ident_str(),
        Log::path_ident_str(),
        LogData::path_ident_str(),
        ScriptResult::path_ident_str(),
        TransferOut::path_ident_str(),
        MessageOut::path_ident_str(),
    ]);
    static ref DISALLOWED_ABI_JSON_TYPES: HashSet<&'static str> = HashSet::from(["Vec"]);
    static ref IGNORED_ABI_JSON_TYPES: HashSet<&'static str> = HashSet::from(["()"]);
    static ref FUEL_PRIMITIVE_RECEIPT_TYPES: HashSet<&'static str> = HashSet::from([
        Transfer::path_ident_str(),
        Log::path_ident_str(),
        LogData::path_ident_str(),
        ScriptResult::path_ident_str(),
        TransferOut::path_ident_str(),
        MessageOut::path_ident_str(),
    ]);
    static ref RUST_PRIMITIVES: HashSet<&'static str> =
        HashSet::from(["u8", "u16", "u32", "u64", "bool", "String"]);
}

fn get_json_abi(abi: String) -> ProgramABI {
    let src = match Source::parse(abi) {
        Ok(src) => src,
        Err(e) => {
            proc_macro_error::abort_call_site!(
                "`abi` must be a file path to valid json abi: {:?}",
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
            "Log" => quote! { Log },
            "BlockData" => quote! { BlockData },
            "LogData" => quote! { LogData },
            "Transfer" => quote! { Transfer },
            "TransferOut" => quote! { TransferOut },
            "ScriptResult" => quote! { ScriptResult },
            "MessageOut" => quote! { MessageOut },
            o if o.starts_with("str[") => quote! { String },
            o => {
                proc_macro_error::abort_call_site!("Unrecognized primitive type: {:?}", o)
            }
        }
    }
}

fn is_fuel_primitive(ty: &proc_macro2::TokenStream) -> bool {
    let ident_str = ty.to_string();
    FUEL_PRIMITIVES.contains(ident_str.as_str())
}

fn is_rust_primitive(ty: &proc_macro2::TokenStream) -> bool {
    let ident_str = ty.to_string();
    RUST_PRIMITIVES.contains(ident_str.as_str())
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
        // TODO: do we want decoder for primitive? Might need something a little smarte to identify what the primitive is for... and to which handler it will go.
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
    manifest: Manifest,
    abi: String,
    input: ItemMod,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    if input.content.is_none() || input.content.as_ref().unwrap().1.is_empty() {
        proc_macro_error::abort_call_site!(
            "No module body, must specify at least one handler function."
        )
    }

    let parsed = get_json_abi(abi);

    let mut abi_types = HashSet::new();
    let mut abi_selectors = Vec::new();
    let mut abi_decoders = Vec::new();
    let mut type_vecs = Vec::new();
    let mut abi_dispatchers = Vec::new();

    let mut type_map = HashMap::new();
    let mut type_ids = FUEL_PRIMITIVES
        .iter()
        .map(|x| (x.to_string(), type_id(FUEL_TYPES_NAMESPACE, x) as usize))
        .collect::<HashMap<String, usize>>();

    for typ in parsed.types {
        if IGNORED_ABI_JSON_TYPES.contains(typ.type_field.as_str()) {
            continue;
        }

        type_map.insert(typ.type_id, typ.clone());

        let ty = rust_type(&typ);
        let name = rust_name(&typ);
        let ty_id = typ.type_id;

        if is_fuel_primitive(&ty) {
            proc_macro_error::abort_call_site!("'{:?} is a reserved Fuel type.")
        }

        type_ids.insert(ty.to_string(), ty_id);

        if !abi_types.contains(&ty_id) {
            type_vecs.push(quote! {
                #name: Vec<#ty>
            });

            abi_decoders.push(decode_snippet(ty_id, &ty, &name));
            abi_types.insert(ty_id);
        }
    }

    for function in parsed.functions {
        let params: Vec<ParamType> = function
            .inputs
            .iter()
            .map(|x| ParamType::try_from_type_application(x, &type_map).unwrap())
            .collect();
        let sig = resolve_fn_selector(&function.name, &params[..]);
        let selector = u64::from_be_bytes(sig);
        let ty_id = function.output.type_id;

        abi_selectors.push(quote! {
            #selector => #ty_id,
        });
    }

    let contents = input.content.unwrap().1;
    let mut handler_fns = Vec::with_capacity(contents.len());

    let mut transfer_decoder = quote! {};
    let mut log_decoder = quote! {};
    let mut blockdata_decoder = quote! {};
    let mut transferout_decoder = quote! {};
    let mut scriptresult_decoder = quote! {};
    let mut messageout_decoder = quote! {};

    let mut blockdata_decoding = quote! {};

    let start_block_conditional = match manifest.start_block {
        Some(start_block) => {
            quote! {
                if block.height < #start_block {
                    continue;
                }
            }
        }
        None => quote! {},
    };

    let contract_conditional = match manifest.contract_id {
        Some(contract_id) => {
            quote! {
                if id != ContractId::from(#contract_id) {
                    continue;
                }
            }
        }
        None => quote! {},
    };

    for item in contents {
        match item {
            Item::Fn(fn_item) => {
                let mut input_checks = Vec::new();
                let mut arg_list = Vec::new();

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
                                            "Type with ident '{:?}' not defined in the ABI.",
                                            path.ident
                                        );
                                    }
                                };

                                if DISALLOWED_ABI_JSON_TYPES
                                    .contains(path_ident_str.as_str())
                                {
                                    proc_macro_error::abort_call_site!(
                                        "Type with ident '{:?}' is not currently supported.",
                                        path.ident
                                    )
                                }

                                if !abi_types.contains(ty_id) {
                                    if FUEL_PRIMITIVES.contains(path_ident_str.as_str()) {
                                        let typ = TypeDeclaration {
                                            type_id: *ty_id,
                                            type_field: path_ident_str.clone(),
                                            type_parameters: None,
                                            components: None,
                                        };

                                        let name = rust_name(&typ);
                                        let ty = rust_type(&typ);

                                        type_vecs.push(quote! {
                                            #name: Vec<#ty>
                                        });

                                        // NOTE: we can't use the generic struct_decoders here because each decoder takes a different
                                        // data param. The generic struct_decoders all take Vec<u8> as their data param while native
                                        // Fuel types take different data params (e.g., Transfer, BlockData, etc)
                                        match path_ident_str.as_str() {
                                            "BlockData" => {
                                                blockdata_decoding = quote! { decoder.decode_blockdata(block.clone()); };
                                                blockdata_decoder =
                                                    quote! { self.#name.push(data); };
                                            }
                                            "Transfer" => {
                                                transfer_decoder =
                                                    quote! { self.#name.push(data); };
                                            }
                                            "TransferOut" => {
                                                transferout_decoder =
                                                    quote! { self.#name.push(data); };
                                            }
                                            "Log" => {
                                                log_decoder =
                                                    quote! { self.#name.push(data); };
                                            }
                                            "LogData" => {
                                                abi_decoders.push(decode_snippet(
                                                    *ty_id, &ty, &name,
                                                ));
                                            }
                                            "ScriptResult" => {
                                                scriptresult_decoder =
                                                    quote! { self.#name.push(data); };
                                            }
                                            "MessageOut" => {
                                                messageout_decoder =
                                                    quote! { self.#name.push(data); };
                                            }
                                            _ => todo!(),
                                        }
                                    } else {
                                        proc_macro_error::abort_call_site!(
                                            "Type with ident '{:?}' not defined in the ABI.",
                                            path.ident,
                                        )
                                    }
                                }

                                let name = rust_name_str(&path.ident.to_string());
                                input_checks.push(quote! { self.#name.len() > 0 });

                                arg_list.push(quote! { self.#name[0].clone() });
                            } else {
                                proc_macro_error::abort_call_site!(
                                    "Arguments must be types defined in the ABI."
                                )
                            }
                        }
                    }
                }

                let fn_name = &fn_item.sig.ident;

                abi_dispatchers.push(quote! {
                    if ( #(#input_checks)&&* ) {
                        #fn_name(#(#arg_list),*);
                    }
                });

                handler_fns.push(fn_item);
            }
            i => {
                proc_macro_error::abort_call_site!(
                    "Unsupported item in indexer module '{:?}'.",
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
                    #(#abi_selectors)*
                    // TODO: should handle this a little more gently
                    _ => panic!("Unknown type id."),
                }
            }

            fn decode_type(&mut self, ty_id: usize, data: Vec<u8>) {
                match ty_id {
                    #(#abi_decoders),*
                    _ => panic!("Unkown type id '{}'.", ty_id),
                }
            }

            pub fn decode_return_type(&mut self, sel: u64, data: Vec<u8>) {
                let ty_id = self.selector_to_type_id(sel);

                self.decode_type(ty_id, data);
            }

            pub fn decode_blockdata(&mut self, data: BlockData) {
                #blockdata_decoder
            }

            pub fn decode_transfer(&mut self, data: Transfer) {
                #transfer_decoder
            }

            pub fn decode_transferout(&mut self, data: TransferOut) {
                #transferout_decoder
            }

            pub fn decode_log(&mut self, data: Log) {
                #log_decoder
            }

            pub fn decode_logdata(&mut self, rb: u64, data: Vec<u8>) {
                // TODO: Use `rb` in `loggedTypes` map in order to find type_id for decoded `data`
                let ty_id = 1;
                self.decode_type(ty_id, data)
            }

            pub fn decode_scriptresult(&mut self, data: ScriptResult) {
                #scriptresult_decoder
            }

            pub fn decode_messageout(&mut self, data: MessageOut) {
                #messageout_decoder
            }

            pub fn dispatch(&self) {
                #(#abi_dispatchers)*
            }
        }
    };
    (
        quote! {
            for block in blocks {

                #start_block_conditional

                let mut decoder = Decoders::default();

                #blockdata_decoding

                for tx in block.transactions {
                    let mut return_types = Vec::new();

                    for receipt in tx {
                        match receipt {
                            Receipt::Call { param1, id, ..} => {
                                #contract_conditional
                                return_types.push(param1);
                            }
                            Receipt::ReturnData { data, id, .. } => {
                                #contract_conditional
                                let selector = return_types.pop().expect("No return type available. <('-'<)");
                                decoder.decode_return_type(selector, data);
                            }
                            Receipt::Transfer { id, to, asset_id, amount, pc, is, .. } => {
                                #contract_conditional
                                let data = Transfer{ contract_id: id, to, asset_id, amount, pc, is };
                                decoder.decode_transfer(data);
                            }
                            Receipt::TransferOut { id, to, asset_id, amount, pc, is, .. } => {
                                #contract_conditional
                                let data = TransferOut{ contract_id: id, to, asset_id, amount, pc, is };
                                decoder.decode_transferout(data);
                            }
                            Receipt::Log { id, ra, rb, .. } => {
                                #contract_conditional
                                let data = Log{ contract_id: id, ra, rb };
                                decoder.decode_log(data);
                            }
                            Receipt::LogData { rb, data, ptr, len, id, .. } => {
                                #contract_conditional
                                decoder.decode_logdata(rb, data);

                            }
                            Receipt::ScriptResult { result, gas_used } => {
                                #contract_conditional
                                let data = ScriptResult{ result: u64::from(result), gas_used };
                                decoder.decode_scriptresult(data);
                            }
                            Receipt::MessageOut { message_id, sender, recipient, amount, nonce, len, digest, data } => {
                                #contract_conditional
                                let payload = MessageOut{ message_id, sender, recipient, amount, nonce, len, digest, data };
                                decoder.decode_messageout(payload);
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

    let IndexerConfig { manifest } = config;

    let path = local_repository_root()
        .map(|x| Path::new(&x).join(&manifest))
        .unwrap_or_else(|| PathBuf::from(&manifest));

    let manifest = Manifest::from_file(&path).unwrap();

    let Manifest {
        abi,
        namespace,
        graphql_schema,
        ..
    } = manifest.clone();

    let indexer = parse_macro_input!(item as ItemMod);

    let (abi_string, schema_string) = match std::env::var("COMPILE_TEST_PREFIX") {
        Ok(prefix) => {
            let prefixed = std::path::Path::new(&prefix).join(&abi);
            let abi_string = prefixed.into_os_string().to_str().unwrap().to_string();
            let prefixed = std::path::Path::new(&prefix).join(&graphql_schema);
            let schema_string = prefixed.into_os_string().to_str().unwrap().to_string();
            (abi_string, schema_string)
        }
        Err(_) => (abi, graphql_schema),
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
            proc_macro_error::abort_call_site!("Could not generate abi object: {:?}", e)
        }
    };

    let graphql_tokens = process_graphql_schema(namespace, schema_string);

    let (handler_block, fn_items) =
        process_fn_items(manifest.clone(), abi_string, indexer);

    let handler_block = if manifest.is_native() {
        handler_block_native(handler_block)
    } else {
        handler_block_wasm(handler_block)
    };

    let output = quote! {
        use alloc::{format, vec, vec::Vec};
        use fuel_indexer_plugin::{types::*, Entity, Logger};
        use fuel_indexer_schema::{serialize, deserialize};
        use fuels_core::{abi_decoder::ABIDecoder, Parameterize, StringToken, Tokenizable};
        use fuel_tx::Receipt;
        use std::collections::HashMap;

        type B256 = [u8; 32];

        #abi_tokens

        #graphql_tokens

        #handler_block

        #fn_items
    };

    proc_macro::TokenStream::from(output)
}
