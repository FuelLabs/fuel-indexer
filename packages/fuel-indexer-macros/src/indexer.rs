use crate::{
    constants::*, helpers::*, native::handler_block_native, parse::IndexerConfig,
    schema::process_graphql_schema, wasm::handler_block_wasm,
};
use fuel_abi_types::abi::program::TypeDeclaration;
use fuel_indexer_lib::{
    graphql::GraphQLSchemaValidator, manifest::ContractIds, manifest::Manifest, type_id,
    utils::local_repository_root, ExecutionSource,
};
use fuel_indexer_types::FUEL_TYPES_NAMESPACE;
use fuels::{core::codec::resolve_fn_selector, types::param_types::ParamType};
use fuels_code_gen::{Abigen, AbigenTarget, ProgramType};
use proc_macro::TokenStream;
use quote::quote;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use syn::{parse_macro_input, FnArg, Item, ItemMod, PatType, Type};

fn process_fn_items(
    manifest: &Manifest,
    abi_path: Option<String>,
    indexer_module: ItemMod,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    if indexer_module.content.is_none()
        || indexer_module
            .content
            .as_ref()
            .expect("Could not parse function input contents.")
            .1
            .is_empty()
    {
        proc_macro_error::abort_call_site!(
            "No module body, must specify at least one handler function."
        )
    }

    let abi = get_json_abi(abi_path);

    let mut decoded_abi_types = HashSet::new();
    let mut abi_dispatchers = Vec::new();

    let funcs = abi.clone().unwrap_or_default().functions;
    let abi_types = abi.clone().unwrap_or_default().types;
    let abi_log_types = abi.clone().unwrap_or_default().logged_types;
    let abi_msg_types = abi.unwrap_or_default().messages_types;
    let fuel_types = FUEL_PRIMITIVES
        .iter()
        .map(|x| {
            let type_id = type_id(FUEL_TYPES_NAMESPACE, x) as usize;
            let typ = TypeDeclaration {
                type_id,
                type_field: x.to_string(),
                components: None,
                type_parameters: None,
            };
            (typ.type_id, typ)
        })
        .collect::<HashMap<usize, TypeDeclaration>>();

    let mut type_ids = FUEL_PRIMITIVES
        .iter()
        .map(|x| (x.to_string(), type_id(FUEL_TYPES_NAMESPACE, x) as usize))
        .collect::<HashMap<String, usize>>();

    let abi_types_tyid = abi_types
        .iter()
        .filter(|typ| {
            if is_ignored_type(typ) {
                return false;
            }
            true
        })
        .map(|typ| (typ.type_id, typ.clone()))
        .collect::<HashMap<usize, TypeDeclaration>>();

    let log_type_decoders = abi_log_types
        .iter()
        .flatten()
        .map(|typ| {
            let ty_id = typ.application.type_id;
            let log_id = typ.log_id as usize;

            quote! {
                #log_id => {
                    self.decode_type(#ty_id, data);
                }
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let message_types_decoders = abi_msg_types
        .iter()
        .flatten()
        .map(|typ| {
            let message_type_id = typ.message_id;
            let ty_id = typ.application.type_id;

            quote! {
                #message_type_id => {
                    self.decode_type(#ty_id, data);
                }
            }
        })
        .chain(
            vec![quote! {
                u64::MAX => {
                    {}
                }
            }]
            .into_iter(),
        )
        .collect::<Vec<proc_macro2::TokenStream>>();

    let abi_type_decoders = abi_types
        .iter()
        .filter_map(|typ| {
            if is_non_decodable_type(typ) {
                return None;
            }

            let name = typ.rust_type_ident();
            let ty = typ.rust_type_token();

            if is_fuel_primitive(&ty) {
                proc_macro_error::abort_call_site!("'{}' is a reserved Fuel type.", ty)
            }

            GraphQLSchemaValidator::check_disallowed_abi_typedef_name(&ty.to_string());

            type_ids.insert(ty.to_string(), typ.type_id);
            decoded_abi_types.insert(typ.type_id);

            Some(decode_snippet(typ.type_id, &ty, &name))
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let fuel_type_decoders = fuel_types
        .values()
        .map(|typ| {
            let name = typ.rust_type_ident();
            let ty = typ.rust_type_token();

            type_ids.insert(ty.to_string(), typ.type_id);
            decoded_abi_types.insert(typ.type_id);

            decode_snippet(typ.type_id, &ty, &name)
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let decoders = [fuel_type_decoders, abi_type_decoders].concat();

    let abi_struct_fields = abi_types
        .iter()
        .filter_map(|typ| {
            if is_non_decodable_type(typ) {
                return None;
            }

            let name = typ.rust_type_ident();
            let ty = typ.rust_type_token();

            if is_fuel_primitive(&ty) {
                proc_macro_error::abort_call_site!("'{}' is a reserved Fuel type.", ty)
            }

            type_ids.insert(ty.to_string(), typ.type_id);
            decoded_abi_types.insert(typ.type_id);

            Some(quote! {
                #name: Vec<#ty>
            })
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let fuel_struct_fields = fuel_types
        .iter()
        .filter_map(|(_ty_id, typ)| {
            if is_non_decodable_type(typ) {
                return None;
            }

            let name = typ.rust_type_ident();
            let ty = typ.rust_type_token();

            type_ids.insert(ty.to_string(), typ.type_id);
            decoded_abi_types.insert(typ.type_id);

            Some(quote! {
                #name: Vec<#ty>
            })
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let decoder_struct_fields = [abi_struct_fields, fuel_struct_fields].concat();

    let abi_selectors = funcs
        .iter()
        .map(|function| {
            let params: Vec<ParamType> = function
                .inputs
                .iter()
                .map(|x| {
                    ParamType::try_from_type_application(x, &abi_types_tyid)
                        .expect("Could not derive TypeApplication param types.")
                })
                .collect();
            let sig = resolve_fn_selector(&function.name, &params[..]);
            let selector = u64::from_be_bytes(sig);
            let ty_id = function.output.type_id;

            quote! {
                #selector => #ty_id,
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let abi_selectors_to_fn_names = funcs
        .iter()
        .map(|function| {
            let params: Vec<ParamType> = function
                .inputs
                .iter()
                .map(|x| {
                    ParamType::try_from_type_application(x, &abi_types_tyid)
                        .expect("Could not derive TypeApplication param types.")
                })
                .collect();
            let sig = resolve_fn_selector(&function.name, &params[..]);
            let fn_name = function.name.clone();
            let selector = u64::from_be_bytes(sig);

            quote! {
               #selector => #fn_name.to_string(),
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let contents = indexer_module
        .content
        .expect("Could not parse input content.")
        .1;

    let mut handler_fns = Vec::with_capacity(contents.len());

    let start_block = match manifest.start_block() {
        Some(start_block) => {
            quote! {
                if block.height < #start_block {
                    continue;
                }
            }
        }
        None => quote! {},
    };

    let subscribed_contract_ids = match &manifest.contract_id() {
        ContractIds::Single(_) => quote! {},
        ContractIds::Multiple(contract_ids) => {
            let contract_ids = contract_ids
                .iter()
                .map(|id| {
                    quote! {
                        let id_bytes = <[u8; 32]>::try_from(#id).expect("Could not convert contract ID into bytes");
                        Bech32ContractId::new("fuel", id_bytes)
                    }
                })
                .collect::<Vec<proc_macro2::TokenStream>>();

            quote! {
                let contract_ids = HashSet::from([#(#contract_ids),*]);
            }
        }
    };

    let check_if_subscribed_to_contract = match &manifest.contract_id() {
        ContractIds::Single(contract_id) => match contract_id {
            Some(contract_id) => {
                quote! {
                    let id_bytes = <[u8; 32]>::try_from(id).expect("Could not convert contract ID into bytes");
                    let bech32_id = Bech32ContractId::new("fuel", id_bytes);
                    let manifest_contract_id = Bech32ContractId::from_str(#contract_id).expect("Failed to parse manifest 'contract_id' as Bech32ContractId");
                    if bech32_id != manifest_contract_id {
                        debug!("Not subscribed to this contract. Will skip this receipt event. <('-'<)");
                        continue;
                    }
                }
            }
            None => quote! {},
        },
        ContractIds::Multiple(_) => {
            quote! {
                let id_bytes = <[u8; 32]>::try_from(id).expect("Could not convert contract ID into bytes");
                let bech32_id = Bech32ContractId::new("fuel", id_bytes);

                if !contract_ids.contains(&bech32_id) {
                    debug!("Not subscribed to this contract. Will skip this receipt event. <('-'<)");
                    continue;
                }
            }
        }
    };

    let (asyncness, awaitness) = manifest.execution_source().async_awaitness();

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
                                let path = path
                                    .path
                                    .segments
                                    .last()
                                    .expect("Could not get last path segment.");

                                let path_ident = path.ident.to_string();
                                let name = decoded_ident(&path_ident);

                                if !type_ids.contains_key(&path_ident) {
                                    proc_macro_error::abort_call_site!(
                                        "Type with ident '{:?}' not defined in the ABI.",
                                        path.ident
                                    );
                                };

                                if DISALLOWED_ABI_JSON_TYPES.contains(path_ident.as_str())
                                {
                                    proc_macro_error::abort_call_site!(
                                        "Type with ident '{:?}' is not currently supported.",
                                        path.ident
                                    )
                                }

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
                        #fn_name(#(#arg_list),*)#awaitness;
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
            #(#decoder_struct_fields),*
        }

        impl Decoders {
            fn selector_to_type_id(&self, sel: u64) -> usize {
                match sel {
                    #(#abi_selectors)*
                    _ => {
                        debug!("Unknown selector; check ABI to make sure function outputs match to types");
                        usize::MAX
                    }
                }
            }

            pub fn selector_to_fn_name(&self, sel: u64) -> String {
                match sel {
                    #(#abi_selectors_to_fn_names)*
                    _ => {
                        debug!("Unknown selector; check ABI to make sure function outputs match to types");
                        "".to_string()
                    }
                }
            }

            fn compute_message_id(&self, sender: &Address, recipient: &Address, nonce: Nonce, amount: Word, data: Option<Vec<u8>>) -> MessageId {

                let mut raw_message_id = Sha256::new()
                    .chain_update(sender)
                    .chain_update(recipient)
                    .chain_update(nonce)
                    .chain_update(amount.to_be_bytes());

                let raw_message_id = if let Some(buffer) = data {
                    raw_message_id
                        .chain_update(&buffer[..])
                        .finalize()
                } else {
                    raw_message_id.finalize()
                };

                let message_id = <[u8; 32]>::try_from(&raw_message_id[..]).expect("Could not calculate message ID from receipt fields");

                message_id.into()
            }

            fn decode_type(&mut self, ty_id: usize, data: Vec<u8>) {
                match ty_id {
                    #(#decoders),*
                    _ => {
                        debug!("Unknown type ID; check ABI to make sure types are correct.");
                    },
                }
            }

            pub fn decode_block(&mut self, data: BlockData) {
                self.blockdata_decoded.push(data);
            }

            pub fn decode_return_type(&mut self, sel: u64, data: Vec<u8>) {
                let ty_id = self.selector_to_type_id(sel);
                self.decode_type(ty_id, data);
            }

            pub fn decode_logdata(&mut self, rb: usize, data: Vec<u8>) {
                match rb {
                    #(#log_type_decoders),*
                    _ => debug!("Unknown logged type ID; check ABI to make sure that logged types are correct.")
                }
            }

            pub fn decode_messagedata(&mut self, type_id: u64, data: Vec<u8>) {
                match type_id {
                    #(#message_types_decoders),*
                    _ => debug!("Unknown message type ID; check ABI to make sure that message types are correct.")
                }
            }

            pub #asyncness fn dispatch(&self) {
                #(#abi_dispatchers)*
            }
        }
    };
    (
        quote! {
            #subscribed_contract_ids

            for block in blocks {

                #start_block

                let mut decoder = Decoders::default();

                let ty_id = BlockData::type_id();
                let data = serialize(&block);
                decoder.decode_type(ty_id, data);

                for tx in block.transactions {

                    let mut return_types = Vec::new();
                    let mut callees = HashSet::new();

                    for receipt in tx.receipts {
                        match receipt {
                            fuel::Receipt::Call { id: contract_id, amount, asset_id, gas, param1, to: id, .. } => {
                                #check_if_subscribed_to_contract

                                let fn_name = decoder.selector_to_fn_name(param1);
                                return_types.push(param1);
                                callees.insert(id);

                                let data = serialize(&Call { contract_id: ContractId::from(<[u8; 32]>::from(contract_id)), to: ContractId::from(<[u8; 32]>::from(id)), amount, asset_id, gas, fn_name });
                                let ty_id = Call::type_id();
                                decoder.decode_type(ty_id, data);
                            }
                            fuel::Receipt::Log { id, ra, rb, .. } => {
                                #check_if_subscribed_to_contract
                                let ty_id = Log::type_id();
                                let data = serialize(&Log{ contract_id: ContractId::from(<[u8; 32]>::from(id)), ra, rb });
                                decoder.decode_type(ty_id, data);
                            }
                            fuel::Receipt::LogData { rb, data, ptr, len, id, .. } => {
                                #check_if_subscribed_to_contract
                                decoder.decode_logdata(rb as usize, data.unwrap_or(Vec::<u8>::new()));
                            }
                            fuel::Receipt::Return { id, val, pc, is } => {
                                #check_if_subscribed_to_contract
                                if callees.contains(&id) {
                                    let ty_id = Return::type_id();
                                    let data = serialize(&Return{ contract_id: ContractId::from(<[u8; 32]>::from(id)), val, pc, is });
                                    decoder.decode_type(ty_id, data);
                                }
                            }
                            fuel::Receipt::ReturnData { data, id, .. } => {
                                #check_if_subscribed_to_contract
                                if callees.contains(&id) {
                                    let selector = return_types.pop().expect("No return type available. <('-'<)");
                                    decoder.decode_return_type(selector, data.unwrap_or(Vec::<u8>::new()));
                                }
                            }
                            fuel::Receipt::MessageOut { sender, recipient, amount, nonce, len, digest, data, .. } => {
                                let sender = Address::from(<[u8; 32]>::from(sender));
                                let recipient = Address::from(<[u8; 32]>::from(recipient));
                                let message_id = decoder.compute_message_id(&sender, &recipient, nonce, amount, data.clone());

                                // It's possible that the data field was generated from an empty Sway `Bytes` array
                                // in the send_message() instruction in which case the data field in the receipt will
                                // have no type information or data to decode, so we decode an empty vector to a unit struct
                                let (type_id, data) = data
                                    .map_or((u64::MAX, Vec::<u8>::new()), |buffer| {
                                        let (type_id_bytes, data_bytes) = buffer.split_at(8);
                                        let type_id = u64::from_be_bytes(
                                            <[u8; 8]>::try_from(type_id_bytes)
                                            .expect("Could not get type ID for data in MessageOut receipt")
                                        );
                                        let data = data_bytes.to_vec();
                                        (type_id, data)
                                    });

                                decoder.decode_messagedata(type_id, data.clone());

                                let ty_id = MessageOut::type_id();
                                let data = serialize(&MessageOut{ message_id, sender, recipient, amount, nonce, len, digest, data });
                                decoder.decode_type(ty_id, data);
                            }
                            fuel::Receipt::ScriptResult { result, gas_used } => {
                                let ty_id = ScriptResult::type_id();
                                let data = serialize(&ScriptResult{ result: u64::from(result), gas_used });
                                decoder.decode_type(ty_id, data);
                            }
                            fuel::Receipt::Transfer { id, to, asset_id, amount, pc, is, .. } => {
                                #check_if_subscribed_to_contract
                                let ty_id = Transfer::type_id();
                                let data = serialize(&Transfer{ contract_id: ContractId::from(<[u8; 32]>::from(id)), to: ContractId::from(<[u8; 32]>::from(to)), asset_id, amount, pc, is });
                                decoder.decode_type(ty_id, data);
                            }
                            fuel::Receipt::TransferOut { id, to, asset_id, amount, pc, is, .. } => {
                                #check_if_subscribed_to_contract
                                let ty_id = TransferOut::type_id();
                                let data = serialize(&TransferOut{ contract_id: ContractId::from(<[u8; 32]>::from(id)), to: Address::from(<[u8; 32]>::from(to)), asset_id, amount, pc, is });
                                decoder.decode_type(ty_id, data);
                            }
                            fuel::Receipt::Panic { id, reason, .. } => {
                                #check_if_subscribed_to_contract
                                let ty_id = Panic::type_id();
                                let data = serialize(&Panic{ contract_id: ContractId::from(<[u8; 32]>::from(id)), reason: *reason.reason() as u32 });
                                decoder.decode_type(ty_id, data);
                            }
                            fuel::Receipt::Revert { id, ra, .. } => {
                                #check_if_subscribed_to_contract
                                let ty_id = Revert::type_id();
                                let data = serialize(&Revert{ contract_id: ContractId::from(<[u8; 32]>::from(id)), error_val: u64::from(ra & 0xF) });
                                decoder.decode_type(ty_id, data);
                            }
                            _ => {
                                info!("This type is not handled yet. (>'.')>");
                            }
                        }
                    }
                }
                decoder.dispatch()#awaitness;

                let metadata = IndexMetadataEntity::new(block.time as u64, block.header.height, block.id);
                metadata.save()#awaitness;
            }
        },
        quote! {
            #decoder_struct

            #(#handler_fns)*
        },
    )
}

pub fn prefix_abi_and_schema_paths(
    abi: Option<&str>,
    schema: &str,
) -> (Option<String>, String) {
    if let Some(abi) = abi {
        match std::env::var("COMPILE_TEST_PREFIX") {
            Ok(prefix) => {
                let prefixed = std::path::Path::new(&prefix).join(abi);
                let abi_string = prefixed
                    .into_os_string()
                    .to_str()
                    .expect("Could not parse prefixed ABI path.")
                    .to_string();
                let prefixed = std::path::Path::new(&prefix).join(schema);
                let schema = prefixed
                    .into_os_string()
                    .to_str()
                    .expect("Could not parse prefixed GraphQL schema path.")
                    .to_string();

                return (Some(abi_string), schema);
            }
            Err(_) => {
                return (Some(abi.into()), schema.to_string());
            }
        };
    }

    (None, schema.to_string())
}

pub fn get_abi_tokens(
    namespace: &str,
    abi: &str,
    exec_source: ExecutionSource,
) -> proc_macro2::TokenStream {
    let no_std = match exec_source {
        ExecutionSource::Native => false,
        ExecutionSource::Wasm => true,
    };

    match Abigen::generate(
        vec![AbigenTarget {
            name: namespace.to_string(),
            abi: abi.to_owned(),
            program_type: ProgramType::Contract,
        }],
        no_std,
    ) {
        Ok(tokens) => tokens,
        Err(e) => {
            proc_macro_error::abort_call_site!(
                "Could not generate tokens for abi: {:?}.",
                e
            )
        }
    }
}

pub fn process_indexer_module(attrs: TokenStream, item: TokenStream) -> TokenStream {
    let config = parse_macro_input!(attrs as IndexerConfig);

    let IndexerConfig { manifest } = config;

    let path = local_repository_root()
        .map(|x| Path::new(&x).join(&manifest))
        .unwrap_or_else(|| PathBuf::from(&manifest));

    let manifest = Manifest::from_file(path).expect("Could not parse manifest.");

    let indexer_module = parse_macro_input!(item as ItemMod);

    let (abi, schema_string) =
        prefix_abi_and_schema_paths(manifest.abi(), manifest.graphql_schema());

    let abi_tokens = match abi {
        Some(ref abi_path) => {
            get_abi_tokens(manifest.namespace(), abi_path, manifest.execution_source())
        }
        None => proc_macro2::TokenStream::new(),
    };

    // NOTE: https://nickb.dev/blog/cargo-workspace-and-the-feature-unification-pitfall/
    let graphql_tokens = process_graphql_schema(
        manifest.namespace(),
        manifest.identifier(),
        &schema_string,
        manifest.execution_source(),
    );

    let output = match manifest.execution_source() {
        ExecutionSource::Native => {
            let (handler_block, fn_items) =
                process_fn_items(&manifest, abi, indexer_module);
            let handler_block = handler_block_native(handler_block);

            quote! {

                #abi_tokens

                #graphql_tokens

                #handler_block

                #fn_items

                #[tokio::main]
                async fn main() -> anyhow::Result<()> {

                    let args = IndexerArgs::parse();

                    let IndexerArgs { manifest, .. } = args.clone();


                    let config = args
                    .config
                    .as_ref()
                    .map(IndexerConfig::from_file)
                    .unwrap_or(Ok(IndexerConfig::from(args)))?;

                    init_logging(&config).await?;

                    info!("Configuration: {:?}", config);

                    let (tx, rx) = channel::<ServiceRequest>(SERVICE_REQUEST_CHANNEL_SIZE);

                    let pool = IndexerConnectionPool::connect(&config.database.to_string()).await?;

                    if config.run_migrations {
                        let mut c = pool.acquire().await?;
                        queries::run_migration(&mut c).await?;
                    }

                    let mut service = IndexerService::new(config.clone(), pool.clone(), rx).await?;

                    if manifest.is_none() {
                        panic!("Manifest required to use native execution.");
                    }

                    let p = manifest.unwrap();
                    if config.verbose {
                        info!("Using manifest file located at '{}'", p.display());
                    }
                    let manifest = Manifest::from_file(&p)?;
                    service.register_native_indexer(manifest, handle_events).await?;

                    let service_handle = tokio::spawn(service.run());
                    let web_handle = tokio::spawn(WebApi::build_and_run(config.clone(), pool, tx));

                    let _ = tokio::join!(service_handle, web_handle);

                    Ok(())
                }
            }
        }
        ExecutionSource::Wasm => {
            let (handler_block, fn_items) =
                process_fn_items(&manifest, abi, indexer_module);
            let handler_block = handler_block_wasm(handler_block);

            quote! {

                #abi_tokens

                #graphql_tokens

                #handler_block

                #fn_items
            }
        }
    };

    proc_macro::TokenStream::from(output)
}
