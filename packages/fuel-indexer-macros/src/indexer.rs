use crate::{
    helpers::*,
    parse::IndexerConfig,
    schema::process_graphql_schema,
    tokens::*,
    wasm::{handler_block, predicate_handler_block},
};
use fuel_abi_types::abi::program::TypeDeclaration;
use fuel_indexer_lib::{
    constants::*, manifest::Manifest, utils::workspace_manifest_prefix,
};
use fuel_indexer_types::{indexer::Predicates, type_id, TypeId, FUEL_TYPES_NAMESPACE};
use fuels::{core::codec::resolve_fn_selector, types::param_types::ParamType};
use fuels_code_gen::{Abigen, AbigenTarget, ProgramType};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use syn::{parse_macro_input, FnArg, Item, ItemMod, PatType, Type};

/// Derive a resultant handler block and a set of handler functions from an indexer manifest and
/// a `TokenStream` of indexer handler functions passed by the user.
fn process_fn_items(
    manifest: &Manifest,
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

    let (contract_abi_path, _schema_path) =
        prefix_abi_and_schema_paths(manifest.contract_abi(), Some(manifest.schema()));

    let contract_abi = get_json_abi(contract_abi_path.as_deref()).unwrap_or_default();

    let mut decoded_type_snippets = HashSet::new();
    let mut decoded_log_match_arms = HashSet::new();
    let mut decoded_type_fields = HashSet::new();
    let mut abi_dispatchers = Vec::new();

    let funcs = contract_abi.clone().functions;
    let contract_abi_types = contract_abi
        .clone()
        .types
        .iter()
        .map(|t| strip_callpath_from_type_field(t.clone()))
        .collect::<Vec<TypeDeclaration>>();

    let predicate_abi_types = manifest
        .predicates()
        .map(|p| {
            p.templates()
                .map(|t| {
                    t.iter()
                        .map(|t| {
                            let ty_id = type_id(FUEL_TYPES_NAMESPACE, &t.name()) as usize;
                            let name = predicate_inputs_name(&t.name());
                            TypeDeclaration {
                                type_id: ty_id,
                                type_field: name,
                                components: Some(Vec::default()),
                                type_parameters: None,
                            }
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let contract_abi_log_types = contract_abi.clone().logged_types.unwrap_or_default();
    let contract_abi_msg_types = contract_abi.clone().messages_types.unwrap_or_default();
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

    let _tuple_types_tyid = contract_abi_types
        .iter()
        .filter_map(|typ| {
            if is_tuple_type(typ) {
                return Some((typ.type_id, typ.clone()));
            }

            None
        })
        .collect::<HashMap<usize, TypeDeclaration>>();

    // Used to do a reverse lookup of typed path names to ABI type IDs.
    let mut contract_abi_type_ids = RESERVED_TYPEDEF_NAMES
        .iter()
        .map(|x| (x.to_string(), type_id(FUEL_TYPES_NAMESPACE, x) as usize))
        .collect::<HashMap<String, usize>>();

    let predicate_abi_type_ids = predicate_abi_types
        .iter()
        .map(|typ| {
            let name = typ.type_field.clone();
            (name, typ.type_id)
        })
        .collect::<HashMap<String, usize>>();

    let mut contract_abi_types_tyid = contract_abi_types
        .iter()
        .map(|typ| (typ.type_id, typ.clone()))
        .collect::<HashMap<usize, TypeDeclaration>>();

    let predicate_abi_types_tyid = predicate_abi_type_ids
        .iter()
        .map(|(name, ty_id)| {
            let typ = TypeDeclaration {
                type_id: *ty_id,
                type_field: name.to_string(),
                components: None,
                type_parameters: None,
            };
            (typ.type_id, typ)
        })
        .collect::<HashMap<usize, TypeDeclaration>>();

    let message_types_decoders = contract_abi_msg_types
        .iter()
        .map(|typ| {
            let message_type_id = typ.message_id;
            let ty_id = typ.application.type_id;

            quote! {
                #message_type_id => {
                    self.decode_type(#ty_id, data)?;
                }
            }
        })
        .chain(vec![quote! {
            u64::MAX => {
                {}
            }
        }])
        .collect::<Vec<proc_macro2::TokenStream>>();

    // Take a second pass over `contract_abi_types` in order to update an `TypeDeclarations` that need to
    // be updated with more information for the codegen process.
    let contract_abi_types = contract_abi_types
        .iter()
        .map(|typ| {
            // If this is an array type we have to manually update it's type field to include its inner
            // type so that the rest of the codegen will work with minimal changes.
            if is_array_type(typ) {
                let inner = typ
                    .components
                    .as_ref()
                    .expect("Array type expects inner components")
                    .first()
                    .expect("Array type expects at least one inner component")
                    .type_id;
                let size = typ
                    .type_field
                    .split(' ')
                    .last()
                    .expect(
                        "Array type has unexpected type field format. Expected [u8; 32]",
                    )
                    .trim_end_matches(']')
                    .parse::<usize>()
                    .expect("Array type size could not be determined.");
                let inner = contract_abi_types_tyid
                    .get(&inner)
                    .expect("Array type inner not found in ABI types.");
                let name = format!("[{}; {}]", inner.name(), size);
                let typ = TypeDeclaration {
                    type_field: name,
                    ..typ.clone()
                };

                contract_abi_types_tyid.insert(typ.type_id, typ.clone());
                typ
            } else {
                typ.to_owned()
            }
        })
        .collect::<Vec<TypeDeclaration>>();

    let contract_abi_decoders = contract_abi_types
        .iter()
        .filter_map(|typ| {
            if is_non_decodable_type(typ) {
                return None;
            }

            if is_fuel_primitive(typ) {
                proc_macro_error::abort_call_site!(
                    "'{}' is a reserved Fuel type.",
                    typ.name()
                )
            }

            if is_generic_type(typ) {
                let gt = GenericType::from(typ);
                match gt {
                    GenericType::Vec | GenericType::Option => {
                        let ab_types = contract_abi_types_tyid.clone();
                        let inner_typs = derive_generic_inner_typedefs(
                            typ,
                            &funcs,
                            &contract_abi_log_types,
                            &ab_types,
                        );

                        return Some(
                            inner_typs
                                .iter()
                                .filter_map(|inner_typ| {
                                    let (typ_name, type_tokens) = typed_path_components(
                                        typ,
                                        inner_typ,
                                        &contract_abi_types_tyid,
                                    );
                                    let ty_id =
                                        type_id(FUEL_TYPES_NAMESPACE, &typ_name) as usize;

                                    let typ = TypeDeclaration {
                                        type_id: ty_id,
                                        type_field: typ_name.clone(),
                                        ..typ.clone()
                                    };

                                    if decoded_type_snippets.contains(&ty_id) {
                                        return None;
                                    }

                                    contract_abi_types_tyid.insert(ty_id, typ.clone());
                                    contract_abi_type_ids.insert(typ_name.clone(), ty_id);

                                    decoded_type_snippets.insert(ty_id);

                                    Some(decode_snippet(&type_tokens, &typ))
                                })
                                .collect::<Vec<proc_macro2::TokenStream>>(),
                        );
                    }
                    _ => unimplemented!("Unsupported decoder generic type: {:?}", gt),
                }
            } else {
                if decoded_type_fields.contains(&typ.type_id) {
                    return None;
                }

                let type_tokens = typ.rust_tokens();
                contract_abi_type_ids.insert(type_tokens.to_string(), typ.type_id);
                decoded_type_snippets.insert(typ.type_id);
                Some(vec![decode_snippet(&type_tokens, typ)])
            }
        })
        .flatten()
        .collect::<Vec<proc_macro2::TokenStream>>();

    let fuel_type_decoders = fuel_types
        .values()
        .filter_map(|typ| {
            if decoded_type_fields.contains(&typ.type_id) {
                return None;
            }

            let type_tokens = typ.rust_tokens();

            contract_abi_type_ids.insert(type_tokens.to_string(), typ.type_id);
            decoded_type_snippets.insert(typ.type_id);

            Some(decode_snippet(&type_tokens, typ))
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let decoders = [fuel_type_decoders, contract_abi_decoders].concat();

    let contract_struct_fields = contract_abi_types
        .iter()
        .filter_map(|typ| {
            if is_non_decodable_type(typ) {
                return None;
            }

            if is_fuel_primitive(typ) {
                proc_macro_error::abort_call_site!(
                    "'{}' is a reserved Fuel type.",
                    typ.name()
                )
            }

            if is_generic_type(typ) {
                let inner_typs = derive_generic_inner_typedefs(
                    typ,
                    &funcs,
                    &contract_abi_log_types,
                    &contract_abi_types_tyid,
                );

                return Some(
                    inner_typs
                        .iter()
                        .filter_map(|inner_typ| {
                            let (typ_name, type_tokens) = typed_path_components(
                                typ,
                                inner_typ,
                                &contract_abi_types_tyid,
                            );
                            let ty_id = type_id(FUEL_TYPES_NAMESPACE, &typ_name) as usize;

                            if decoded_type_fields.contains(&ty_id) {
                                return None;
                            }

                            let typ = TypeDeclaration {
                                type_id: ty_id,
                                type_field: typ_name.clone(),
                                ..typ.clone()
                            };

                            let ident = typ.decoder_field_ident();

                            contract_abi_type_ids.insert(typ_name.clone(), ty_id);
                            decoded_type_fields.insert(ty_id);

                            Some(quote! {
                                #ident: Vec<#type_tokens>
                            })
                        })
                        .collect::<Vec<proc_macro2::TokenStream>>(),
                );
            } else {
                if decoded_type_fields.contains(&typ.type_id) {
                    return None;
                }

                let ident = typ.decoder_field_ident();
                let type_tokens = typ.rust_tokens();
                contract_abi_type_ids.insert(typ.rust_tokens().to_string(), typ.type_id);
                decoded_type_fields.insert(typ.type_id);

                Some(vec![quote! {
                    #ident: Vec<#type_tokens>
                }])
            }
        })
        .flatten()
        .collect::<Vec<proc_macro2::TokenStream>>();

    let fuel_struct_fields = fuel_types
        .iter()
        .filter_map(|(_ty_id, typ)| {
            if is_non_decodable_type(typ) {
                return None;
            }

            if decoded_type_fields.contains(&typ.type_id) {
                return None;
            }

            let name = typ.decoder_field_ident();
            let ty = typ.rust_tokens();

            contract_abi_type_ids.insert(ty.to_string(), typ.type_id);
            decoded_type_snippets.insert(typ.type_id);

            if typ.type_id == Predicates::type_id() {
                Some(quote! {
                    #name: Predicates
                })
            } else {
                Some(quote! {
                    #name: Vec<#ty>
                })
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let predicate_inputs_fields = manifest
        .predicates()
        .map(|p| {
            p.templates()
                .map(|t| {
                    t.iter()
                        .map(|t| {
                            let name = predicate_inputs_name(&t.name());
                            let ident =
                                format_ident! { "{}_decoded", name.to_lowercase() };
                            let ty = format_ident! { "{}", name };
                            let ty = quote! { #ty };

                            quote! {
                                #ident: Vec<#ty>
                            }
                        })
                        .collect::<Vec<proc_macro2::TokenStream>>()
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let decoder_fields = [
        contract_struct_fields,
        fuel_struct_fields,
        predicate_inputs_fields,
    ]
    .concat();

    // Since log type decoders use `TypeDeclaration`s that were manually created specifically
    // for generics, we parsed log types after other ABI types.
    let contract_log_type_decoders = contract_abi_log_types
        .iter()
        .filter_map(|log| {
            let ty_id = log.application.type_id;
            let log_id = log.log_id as usize;
            let typ = contract_abi_types_tyid
                .get(&log.application.type_id)
                .expect("Could not get log type reference from ABI types.");

            if is_non_decodable_type(typ) {
                return None;
            }

            if is_generic_type(typ) {
                let gt = GenericType::from(typ);
                match gt {
                    GenericType::Vec | GenericType::Option => {
                        let inner_typ = derive_log_generic_inner_typedefs(
                            log,
                            &contract_abi_log_types,
                            &contract_abi_types_tyid,
                        );

                        let (typ_name, _) = typed_path_components(
                            typ,
                            inner_typ,
                            &contract_abi_types_tyid,
                        );

                        let ty_id = type_id(FUEL_TYPES_NAMESPACE, &typ_name) as usize;
                        let _typ = contract_abi_types_tyid.get(&ty_id).expect(
                            "Could not get generic log type reference from ABI types.",
                        );

                        decoded_log_match_arms.insert(log_id);

                        Some(quote! {
                            #log_id => {
                                self.decode_type(#ty_id, data)?;
                            }
                        })
                    }
                    _ => unimplemented!("Unsupported decoder generic type: {:?}", gt),
                }
            } else {
                decoded_log_match_arms.insert(log_id);

                Some(quote! {
                    #log_id => {
                        self.decode_type(#ty_id, data)?;
                    }
                })
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let contract_abi_selectors = funcs
        .iter()
        .map(|function| {
            let params: Vec<ParamType> = function
                .inputs
                .iter()
                .map(|x| {
                    ParamType::try_from_type_application(x, &contract_abi_types_tyid)
                        .expect("Could not derive TypeApplication param types.")
                })
                .collect();
            let sig = resolve_fn_selector(&function.name, &params[..]);
            let selector = u64::from_be_bytes(sig);
            let ty_id = function_output_type_id(function, &contract_abi_types_tyid);

            quote! {
                #selector => #ty_id,
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let contract_abi_selectors_to_fn_names = funcs
        .iter()
        .map(|function| {
            let params: Vec<ParamType> = function
                .inputs
                .iter()
                .map(|x| {
                    ParamType::try_from_type_application(x, &contract_abi_types_tyid)
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
                    return Ok(());
                }
            }
        }
        None => quote! {},
    };

    let subscribed_contract_ids = match &manifest.contract_subscriptions() {
        Some(ids) => {
            let contract_ids = ids.iter().map(|id| {
                quote! {
                    Bech32ContractId::from_str(#id).expect("Failed to parse contract ID from manifest.")
                }
            }).collect::<Vec<proc_macro2::TokenStream>>();

            quote! {
                let contract_ids = HashSet::from([#(#contract_ids),*]);
            }
        }
        None => quote! {},
    };

    let check_if_subscribed_to_contract = match &manifest.contract_subscriptions() {
        Some(_) => {
            quote! {
                let id_bytes = <[u8; 32]>::try_from(id).expect("Could not convert contract ID into bytes");
                let bech32_id = Bech32ContractId::new("fuel", id_bytes);

                if !contract_ids.contains(&bech32_id) {
                    debug!("Not subscribed to this contract. Will skip this receipt event. <('-'<)");
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
                                let path_seg = path
                                    .path
                                    .segments
                                    .last()
                                    .expect("Could not get last path segment.");

                                let path_type_name = typed_path_name(path);

                                if is_unsupported_type(&path_type_name) {
                                    proc_macro_error::abort_call_site!(
                                        "Type with ident '{:?}' is not currently supported.",
                                        path_seg.ident
                                    )
                                }

                                if !contract_abi_type_ids.contains_key(&path_type_name)
                                    && !predicate_abi_type_ids
                                        .contains_key(&path_type_name)
                                {
                                    proc_macro_error::abort_call_site!(
                                        "Type with ident '{:?}' not defined in the ABI.",
                                        path_seg.ident
                                    );
                                };

                                let ty_id =
                                    match contract_abi_type_ids.get(&path_type_name) {
                                        Some(ty_id) => ty_id,
                                        None => predicate_abi_type_ids
                                            .get(&path_type_name)
                                            .expect("Type not found in Fuel types."),
                                    };

                                let typ = match contract_abi_types_tyid.get(ty_id) {
                                    Some(typ) => typ,
                                    None => match fuel_types.get(ty_id) {
                                        Some(typ) => typ,
                                        None => {
                                            match predicate_abi_types_tyid.get(ty_id) {
                                                Some(typ) => typ,
                                                None => {
                                                    panic!("Type not found in ABI types.")
                                                }
                                            }
                                        }
                                    },
                                };

                                let dispatcher_name = typ.decoder_field_ident();

                                input_checks
                                    .push(quote! { self.#dispatcher_name.len() > 0 });

                                arg_list.push(dispatcher_tokens(&dispatcher_name));
                            } else {
                                proc_macro_error::abort_call_site!(
                                    "Arguments must be types defined in the ABI."
                                )
                            }
                        }
                    }
                }

                let fn_name = &fn_item.sig.ident;
                let fn_name_string = fn_name.to_string();

                if arg_list.is_empty() {
                    proc_macro_error::abort_call_site!(
                        "Handler function '{}' must have at least one argument.",
                        fn_name.to_string(),
                    );
                }

                let fn_call = if fn_item.sig.output == syn::ReturnType::Default {
                    quote! {
                        #fn_name(#(#arg_list),*)
                    }
                } else {
                    quote! {
                        if let Err(e) = #fn_name(#(#arg_list),*).with_context(|| format!("Failed executing {}()", #fn_name_string)) {
                            unsafe {
                                if !ERROR_MESSAGE.is_empty() {
                                    ERROR_MESSAGE += "\n\n";
                                }
                                // Piggyback on ERROR_MESSAGE to collect multiple errors.
                                // ERROR_MESSAGE is converted back to Error at the end of dispatch()
                                ERROR_MESSAGE += &format!("{e:?}");
                            }
                        }
                    }
                };

                abi_dispatchers.push(quote! {
                    if ( #(#input_checks)&&* ) {
                        #fn_call
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

    let predicate_tokens = transaction_predicate_tokens(manifest);
    let predicate_block = predicate_handler_block(manifest, predicate_tokens);
    let predicate_decoder_fns = predicate_decoder_fn_tokens(manifest);

    let decoder_struct = decoder_struct_tokens(
        decoder_fields,
        contract_abi_selectors,
        contract_abi_selectors_to_fn_names,
        decoders,
        contract_log_type_decoders,
        message_types_decoders,
        abi_dispatchers,
        predicate_decoder_fns,
    );

    let process_transaction =
        process_transaction_tokens(check_if_subscribed_to_contract, predicate_block);

    (
        quote! {
            #subscribed_contract_ids

            #process_transaction

            let mut process_block = |block: BlockData| -> anyhow::Result<()> {
                #start_block

                let mut decoder = Decoders::default();

                let ty_id = BlockData::type_id();
                let data = serialize(&block);
                decoder.decode_type(ty_id, data)?;

                for tx in block.transactions {
                    let tx_id = tx.id;
                    process_transaction(&mut decoder, tx).with_context(|| format!("Failed processing Transaction {:?}", tx_id))?
                }

                decoder.dispatch()?;

                let metadata = IndexMetadataEntity::new(block.time as u64, block.header.height, block.id);
                metadata.save();
                Ok(())
            };

            for block in blocks {
                let block_height = block.header.height;
                process_block(block).with_context(|| format!("Failed processing Block #{}", block_height))?;
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

    let path = workspace_manifest_prefix()
        .map(|x| Path::new(&x).join(&manifest))
        .unwrap_or_else(|| PathBuf::from(&manifest));

    let manifest = Manifest::from_file(path).expect("Could not parse manifest.");

    let indexer_module = parse_macro_input!(item as ItemMod);

    let predicate_abi_info = manifest
        .predicates()
        .map(|p| {
            p.templates()
                .map(|t| {
                    t.iter()
                        .filter_map(|t| {
                            let (predicate_abi, _) = prefix_abi_and_schema_paths(
                                Some(t.abi()),
                                Some(manifest.schema()),
                            );

                            if let Some(predicate_abi) = predicate_abi {
                                return Some((predicate_abi, t.name()));
                            }
                            None
                        })
                        .collect::<Vec<(String, String)>>()
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let mut targets = Vec::new();

    if let Some(ref abi) = manifest.contract_abi() {
        targets.push(AbigenTarget {
            // FIXME: We should be using the name of the contract here
            name: manifest.namespace().to_string(),
            abi: abi.to_string(),
            program_type: ProgramType::Contract,
        });
    }

    predicate_abi_info
        .iter()
        .for_each(|(abi_path, template_name)| {
            targets.push(AbigenTarget {
                name: template_name.to_string(),
                abi: abi_path.to_string(),
                program_type: ProgramType::Predicate,
            });
        });

    let abi_tokens = match Abigen::generate(targets, true) {
        Ok(tokens) => tokens,
        Err(e) => {
            proc_macro_error::abort_call_site!(
                "Could not generate tokens for ABI: {:?}.",
                e
            )
        }
    };

    // NOTE: https://nickb.dev/blog/cargo-workspace-and-the-feature-unification-pitfall/
    let graphql_tokens = process_graphql_schema(
        manifest.namespace(),
        manifest.identifier(),
        manifest.schema(),
    );

    let predicate_impl_tokens = predicate_entity_impl_tokens();

    let (block, fn_items) = process_fn_items(&manifest, indexer_module);
    let block = handler_block(block, fn_items);
    let predicate_inputs = predicate_inputs_tokens(&manifest);

    let output = quote! {

        #predicate_inputs

        #abi_tokens

        #graphql_tokens

        #predicate_impl_tokens

        #block
    };

    proc_macro::TokenStream::from(output)
}
