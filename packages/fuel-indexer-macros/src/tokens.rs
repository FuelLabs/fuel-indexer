/// Various functions that generate TokenStreams used to augment the indexer ASTs.
use crate::helpers::*;
use fuel_abi_types::abi::program::TypeDeclaration;
use fuel_indexer_lib::manifest::Manifest;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::Ident;

/// `TokenStream` that represents the `Decoder` from which all incoming data to the transaction handler
/// is decoded, and all indexer handlers are dispatched.
#[allow(clippy::too_many_arguments)]
pub fn decoder_struct_tokens(
    decoder_fields: Vec<TokenStream>,
    abi_selectors: Vec<TokenStream>,
    abi_selectors_to_fn_names: Vec<TokenStream>,
    decoders: Vec<TokenStream>,
    contract_log_decoders: Vec<TokenStream>,
    message_decoders: Vec<TokenStream>,
    abi_dispatchers: Vec<TokenStream>,
    predicate_decoder_fns: TokenStream,
) -> TokenStream {
    quote! {
        #[derive(Default)]
        struct Decoders {
            #(#decoder_fields),*
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
                        String::new()
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

            fn decode_type(&mut self, ty_id: usize, data: Vec<u8>) -> anyhow::Result<()> {
                let decoder = ABIDecoder::default();
                match ty_id {
                    #(#decoders),*
                    _ => {
                        debug!("Unknown type ID; check ABI to make sure types are correct.");
                    },
                }
                Ok(())
            }

            pub fn decode_predicate(&mut self, data: Vec<u8>, template_id: &str) -> anyhow::Result<()> {
                let predicate: IndexerPredicate = bincode::deserialize(&data)
                    .expect("Could not deserialize predicate from bytes");
                self.predicates_decoded.add(template_id.to_string(), predicate);
                Ok(())
            }

            #predicate_decoder_fns

            pub fn decode_block(&mut self, data: BlockData) {
                self.blockdata_decoded.push(data);
            }

            pub fn decode_return_type(&mut self, sel: u64, data: Vec<u8>) -> anyhow::Result<()> {
                let ty_id = self.selector_to_type_id(sel);
                self.decode_type(ty_id, data)?;
                Ok(())
            }

            pub fn decode_logdata(&mut self, rb: usize, data: Vec<u8>) -> anyhow::Result<()> {
                match rb {
                    #(#contract_log_decoders),*
                    _ => debug!("Unknown logged type ID; check ABI to make sure that logged types are correct.")
                }
                Ok(())
            }

            pub fn decode_messagedata(&mut self, type_id: u64, data: Vec<u8>) -> anyhow::Result<()> {
                match type_id {
                    #(#message_decoders),*
                    _ => debug!("Unknown message type ID; check ABI to make sure that message types are correct.")
                }
                Ok(())
            }

            pub fn dispatch(&self) -> anyhow::Result<()> {
                #(#abi_dispatchers)*

                unsafe {
                    if !ERROR_MESSAGE.is_empty() {
                        anyhow::bail!(ERROR_MESSAGE.clone());
                    } else {
                        Ok(())
                    }
                }
            }
        }
    }
}

/// `TokenStream` used to build the transaction handler in the `handle_events` entrypoint.
///
/// This triggers the `Decoder` based on the supplied inputs (i.e., the outputs received from Fuel client).
pub fn process_transaction_tokens(
    contract_subscription: TokenStream,
    predicate_block: TokenStream,
) -> TokenStream {
    quote! {
        let mut process_transaction = |decoder: &mut Decoders, tx: fuel::TransactionData| -> anyhow::Result<()> {
            let tx_id = tx.id;

            #predicate_block

            let mut return_types = Vec::new();
            let mut callees = HashSet::new();

            for receipt in tx.receipts {
                match receipt {
                    fuel::Receipt::Call { id: contract_id, amount, asset_id, gas, param1, to: id, .. } => {
                        #contract_subscription

                        let fn_name = decoder.selector_to_fn_name(param1);
                        return_types.push(param1);
                        callees.insert(id);

                        let data = serialize(
                            &Call {
                                contract_id: ContractId::from(<[u8; 32]>::from(contract_id)),
                                to: ContractId::from(<[u8; 32]>::from(id)),
                                amount,
                                asset_id: AssetId::from(<[u8; 32]>::from(asset_id)),
                                gas,
                                fn_name
                            }
                        );
                        let ty_id = Call::type_id();
                        decoder.decode_type(ty_id, data)?;
                    }
                    fuel::Receipt::Log { id, ra, rb, .. } => {
                        #contract_subscription
                        let ty_id = Log::type_id();
                        let data = serialize(
                            &Log {
                                contract_id: ContractId::from(<[u8; 32]>::from(id)),
                                ra,
                                rb
                            }
                        );
                        decoder.decode_type(ty_id, data)?;
                    }
                    fuel::Receipt::LogData { rb, data, ptr, len, id, .. } => {
                        #contract_subscription
                        decoder.decode_logdata(rb as usize, data.unwrap_or(Vec::<u8>::new()))?;
                    }
                    fuel::Receipt::Return { id, val, pc, is } => {
                        #contract_subscription
                        if callees.contains(&id) {
                            let ty_id = Return::type_id();
                            let data = serialize(
                                &Return {
                                    contract_id: ContractId::from(<[u8; 32]>::from(id)),
                                    val,
                                    pc,
                                    is
                                }
                            );
                            decoder.decode_type(ty_id, data)?;
                        }
                    }
                    fuel::Receipt::ReturnData { data, id, .. } => {
                        #contract_subscription
                        if callees.contains(&id) {
                            let selector = return_types.pop().expect("No return type available. <('-'<)");
                            decoder.decode_return_type(selector, data.unwrap_or(Vec::<u8>::new()))?;
                        }
                    }
                    fuel::Receipt::MessageOut { sender, recipient, amount, nonce, len, digest, data, .. } => {
                        let sender = Address::from(<[u8; 32]>::from(sender));
                        let recipient = Address::from(<[u8; 32]>::from(recipient));
                        let message_id = decoder.compute_message_id(&sender, &recipient, nonce, amount, data.clone());

                        // It's possible that the data field was generated from an empty Sway `Bytes` array
                        // in the send_message() instruction in which case the data field in the receipt will
                        // have no type information or data to decode. Thus, we check for a None value or
                        // an empty byte vector; if either condition is present, then we decode to a unit struct instead.
                        let (type_id, data) = data
                            .map_or((u64::MAX, Vec::<u8>::new()), |buffer| {
                                if buffer.is_empty() {
                                    (u64::MAX, Vec::<u8>::new())
                                } else {
                                    let (type_id_bytes, data_bytes) = buffer.split_at(8);
                                    let type_id = u64::from_be_bytes(
                                        <[u8; 8]>::try_from(type_id_bytes)
                                        .expect("Could not get type ID for data in MessageOut receipt")
                                    );
                                    let data = data_bytes.to_vec();
                                    (type_id, data)
                                }
                            });

                        decoder.decode_messagedata(type_id, data.clone())?;

                        let ty_id = MessageOut::type_id();
                        let data = serialize(
                            &MessageOut {
                                message_id,
                                sender,
                                recipient,
                                amount,
                                nonce,
                                len,
                                digest,
                                data
                            }
                        );
                        decoder.decode_type(ty_id, data)?;
                    }
                    fuel::Receipt::ScriptResult { result, gas_used } => {
                        let ty_id = ScriptResult::type_id();
                        let data = serialize(&ScriptResult{ result: u64::from(result), gas_used });
                        decoder.decode_type(ty_id, data)?;
                    }
                    fuel::Receipt::Transfer { id, to, asset_id, amount, pc, is, .. } => {
                        #contract_subscription
                        let ty_id = Transfer::type_id();
                        let data = serialize(
                            &Transfer {
                                contract_id: ContractId::from(<[u8; 32]>::from(id)),
                                to: ContractId::from(<[u8; 32]>::from(to)),
                                asset_id: AssetId::from(<[u8; 32]>::from(asset_id)),
                                amount,
                                pc,
                                is
                            }
                        );
                        decoder.decode_type(ty_id, data)?;
                    }
                    fuel::Receipt::TransferOut { id, to, asset_id, amount, pc, is, .. } => {
                        #contract_subscription
                        let ty_id = TransferOut::type_id();
                        let data = serialize(
                            &TransferOut {
                                contract_id: ContractId::from(<[u8; 32]>::from(id)),
                                to: Address::from(<[u8; 32]>::from(to)),
                                asset_id: AssetId::from(<[u8; 32]>::from(asset_id)),
                                amount,
                                pc,
                                is
                            }
                        );
                        decoder.decode_type(ty_id, data)?;
                    }
                    fuel::Receipt::Panic { id, reason, .. } => {
                        #contract_subscription
                        let ty_id = Panic::type_id();
                        let data = serialize(
                            &Panic {
                                contract_id: ContractId::from(<[u8; 32]>::from(id)),
                                reason: *reason.reason() as u32
                            }
                        );
                        decoder.decode_type(ty_id, data)?;
                    }
                    fuel::Receipt::Revert { id, ra, .. } => {
                        #contract_subscription
                        let ty_id = Revert::type_id();
                        let data = serialize(
                            &Revert {
                                contract_id: ContractId::from(<[u8; 32]>::from(id)),
                                error_val: u64::from(ra & 0xF)
                            }
                        );
                        decoder.decode_type(ty_id, data)?;
                    }
                    fuel::Receipt::Mint { sub_id, contract_id, val, pc, is } => {
                        let ty_id = Mint::type_id();
                        let data = serialize(
                            &Mint {
                                sub_id: AssetId::from(<[u8; 32]>::from(sub_id)),
                                contract_id: ContractId::from(<[u8; 32]>::from(contract_id)),
                                val,
                                pc,
                                is
                            }
                        );
                        decoder.decode_type(ty_id, data)?;
                    }
                    fuel::Receipt::Burn { sub_id, contract_id, val, pc, is } => {
                        let ty_id = Burn::type_id();
                        let data = serialize(
                            &Burn {
                                sub_id: AssetId::from(<[u8; 32]>::from(sub_id)),
                                contract_id: ContractId::from(<[u8; 32]>::from(contract_id)),
                                val,
                                pc,
                                is
                            }
                        );
                        decoder.decode_type(ty_id, data)?;
                    }
                    _ => {
                        info!("This type is not handled yet. (>'.')>");
                    }
                }
            }

            Ok(())
        };
    }
}

/// `TokenStream` used to augment the `process_transaction_tokens` `TokenStream` with logic
/// specific to the handling of predicates in the `handle_events` entrypoint.
///
/// If predicates are not enabled, the AST will not be augmented with these tokens.
pub fn transaction_predicate_tokens(manifest: &Manifest) -> TokenStream {
    let configurable_names = predicate_inputs_names_map(manifest);
    let verification_tokens = manifest
        .predicates()
        .map(|p| {
            p.templates()
                .map(|t| {
                    t.iter().map(|t| {

                        let inputs_name =
                            format_ident! { "{}", predicate_inputs_name(&t.name()) };
                        let configurables_name =
                            format_ident! { "{}", configurables_name(&t.name()) };

                        let abi = get_json_abi(Some(t.abi()))
                            .expect("Could not derive predicate JSON ABI.");

                        let abi_types = abi
                            .types
                            .iter()
                            .map(|typ| (typ.type_id, typ.clone()))
                            .collect::<HashMap<usize, TypeDeclaration>>();

                        let inputs_fields = abi
                            .configurables
                            .iter()
                            .flatten()
                            .map(|c| {
                                let ty_id = c.application.type_id;
                                let name = configurable_names
                                    .get(&ty_id)
                                    .expect("Could not find configurable naming.");

                                format_ident! {"{}", name }
                            })
                            .collect::<Vec<_>>();

                        let chained_encoder_funcs = abi.configurables
                            .iter()
                            .flatten()
                            .map(|configurable| {
                                let typ = abi_types.get(&configurable.application.type_id).expect(
                                    "Predicate configurable type not found in the ABI.",
                                );

                                let clone = if is_copy_type(typ) {
                                    quote! { .clone() }
                                } else {
                                    quote! {}
                                };

                                let ty_name = configurable_fn_type_name(configurable)
                                    .expect("Cannot use unit types '()' in configurables.");
                                let arg = configurable_names
                                    .get(&configurable.application.type_id)
                                    .expect("Could not find configurable naming.");
                                let arg = format_ident! {"{}", arg };
                                let fn_name = format_ident! { "with_{}", ty_name };
                                quote! {
                                    .#fn_name(#arg #clone)
                                }
                            });

                        quote! {
                            PredicatesInputs::#inputs_name(#inputs_name { #(#inputs_fields),*, .. }) => {
                                let configurables = #configurables_name::new()#(#chained_encoder_funcs)*;
                                let mut predicate = SDKPredicate::from_code(indexer_predicate.bytecode().clone(), BETA4_CHAIN_ID)
                                    .with_configurables(configurables);
                                // Convert the SDK predicate to the indexer-friendly predicate
                                let predicate = Predicate::from(predicate);

                                if predicate.address() == indexer_predicate.coin_output().owner() {
                                    // IndexerPredicateEntity will automatically save the PredicateCoinOutputEntity
                                    let predicate_entity = IndexerPredicateEntity::from(indexer_predicate.clone());
                                    predicate_entity.save();
                                }
                            }
                        }
            }).collect::<Vec<proc_macro2::TokenStream>>()
        }).unwrap_or_default()
    }).unwrap_or_default();

    let decode_inputs_tokens = manifest
        .predicates()
        .map(|p| {
            p.templates()
                .map(|t| {
                    t.iter().map(|t| {
                        let name = predicate_inputs_name(&t.name());
                        let template_id = t.id().to_string();
                        let ident = format_ident! { "{}", name };
                        let fn_name = format_ident! { "decode_{}", name.to_lowercase() };

                        quote! {
                            #template_id => {
                                let obj = #ident::new(predicate_data.to_owned());
                                decoder.#fn_name(obj).expect("Could not decode predicate input.");
                            },
                        }
                    }).collect::<Vec<proc_macro2::TokenStream>>()
                }).unwrap_or_default()
        }).unwrap_or_default();

    let inputs_match_tokens = manifest
        .predicates()
        .map(|p| {
            p.templates()
                .map(|t| {
                    t.iter()
                        .map(|t| {
                            let template_id = t.id().to_string();
                            let name = predicate_inputs_name(&t.name());
                            let ident = format_ident! { "{}", name };
                            quote! {
                                #template_id => {
                                    let obj = #ident::new(configurables);
                                    Some(PredicatesInputs::#ident(obj))
                                },
                            }
                        })
                        .collect::<Vec<proc_macro2::TokenStream>>()
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();

    let template_ids = manifest
        .predicates()
        .map(|p| {
            p.templates()
                .map(|t| {
                    t.iter()
                        .map(|t| {
                            let template_id = t.id().to_string();
                            quote! { #template_id.to_string() }
                        })
                        .collect::<Vec<proc_macro2::TokenStream>>()
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();

    quote! {
        match &tx.transaction {
            // Part 1: Finding and saving any submitted predicates in this transaction
            fuel::Transaction::Script(script) => {
                let fuel::Script {
                    inputs,
                    outputs,
                    witnesses,
                    ..
                } = script;

                let template_ids = vec![#(#template_ids),*];

                let submitted_predicates = witnesses
                    .iter()
                    .map(|w| PredicateWitnessData::try_from(w.to_owned()))
                    .filter_map(Result::ok)
                    .filter_map(|pwd| {
                        let template_id = pwd.template_id().to_string();
                        if template_ids.contains(&template_id) {
                            Some(pwd)
                        } else {
                            None
                        }
                    })
                    .map(|data| {
                        IndexerPredicate::from_witness(
                            data.to_owned(),
                            tx_id,
                            outputs[data.output_index() as usize].to_owned(),
                        )
                    })
                    .collect::<Vec<_>>();

                submitted_predicates.iter().for_each(|indexer_predicate| {
                    let template_id = indexer_predicate.template_id().to_string();
                    let configurables = indexer_predicate.configurables().to_owned();
                    let predicate_inputs = match template_id.as_str() {
                        #(#inputs_match_tokens)*
                        _ => {
                            Logger::error("Unknown predicate template ID; check ABI to make sure that predicate IDs are correct.");
                            None
                        }
                    };


                    if let Some(predicate_inputs) = predicate_inputs {
                        match predicate_inputs {
                            #(#verification_tokens)*
                            _ => Logger::error("Unrecognized configurable type."),
                        }

                    }
                });

                // Part 2: Finding any spent predicates in this transaction
                inputs.iter().for_each(|i| {
                    match i {
                        fuel::Input::Coin(coin) => {
                            let fuel::InputCoin {
                                utxo_id,
                                owner,
                                amount,
                                asset_id,
                                predicate: predicate_code,
                                predicate_data,
                                ..
                            } = coin;

                            let utxos = PredicateCoinOutputEntity::find_many(PredicateCoinOutputEntity::owner().eq(owner.to_owned()));
                            utxos.iter().for_each(|utxo| {
                                if let Some(predicate_entity) = IndexerPredicateEntity::find(IndexerPredicateEntity::coin_output().eq(utxo.id.clone())) {
                                    let indexer_predicate = IndexerPredicate::from(predicate_entity);
                                    let template_id = indexer_predicate.template_id().to_string();
                                    let data = bincode::serialize(&indexer_predicate).expect("Could not serialize predicate.");
                                    decoder.decode_predicate(data, template_id.as_str()).expect("Could not decode predicate.");

                                    match template_id.as_str() {
                                        #(#decode_inputs_tokens)*
                                        _ => {
                                            Logger::error("Unknown predicate template ID; check ABI to make sure that predicate IDs are correct.");
                                        }
                                    }
                                }
                            });
                        }
                        _ => {
                            debug!("Unsupported predicate input type.");
                        }
                    }
                });
            }
            _ => {
                debug!("Unsupported predicate transaction type.");
            }
        }
    }
}

/// `TokenStream` used to generate a set of indexer-specific predicate inputs.
pub fn predicate_inputs_tokens(manifest: &Manifest) -> TokenStream {
    let predicates = manifest.predicates();

    let mut output = quote! {
        trait InputDecoder {
            fn decode_type(&self, ty_id: usize, data: Vec<u8>) -> anyhow::Result<()>;
        }
    };

    if let Some(p) = predicates {
        if p.is_empty() {
            return output;
        }

        let inputs_namings = predicate_inputs_names_map(manifest);

        let mut configurable_variants = Vec::new();

        let names = p
            .templates()
            .map(|t| t.iter().map(|t| t.name()).collect::<Vec<_>>())
            .unwrap_or_default();
        let paths = p
            .templates()
            .map(|t| t.iter().map(|t| t.abi()).collect::<Vec<_>>())
            .unwrap_or_default();

        names.into_iter().zip(paths).for_each(|(name, abi_path)| {
            let abi = get_json_abi(Some(abi_path))
                .expect("Could not derive predicate JSON ABI.");

            let predicate_types = abi
                .types
                .iter()
                .map(|typ| (typ.type_id, typ.clone()))
                .collect::<HashMap<usize, TypeDeclaration>>();

            let main = abi
                .functions
                .iter()
                .find(|f| f.name == "main")
                .expect("ABI missing main function.");

            let inputs_struct_fields = main.inputs.iter().map(|input| {
                let ty_id = input.type_id;
                let typ = predicate_types.get(&ty_id).expect("Could not find type in ABI.");
                let name = inputs_namings
                    .get(&ty_id)
                    .expect("Could not find configurable naming.");
                let ty = typ.rust_tokens();
                let name = format_ident! { "{}", name };

                quote! {
                    #name: #ty
                }
            }).collect::<Vec<_>>();

            let mut predicate_data_token_decoding = Vec::new();
            let mut inputs_struct_constructor = Vec::new();

            let inputs_param_types = main.inputs.iter().map(|input| {
                let ty_id = input.type_id;
                let typ = predicate_types.get(&ty_id).expect("Could not find type in ABI.");
                let ty = typ.rust_tokens();

                quote! {
                    #ty::param_type()
                }
            }).collect::<Vec<_>>();

            main.inputs.iter().for_each(|input| {
                let ty_id = input.type_id;
                let typ = predicate_types
                    .get(&ty_id)
                    .expect("Predicate configurable type not found in the ABI.");
                let name = inputs_namings
                    .get(&ty_id)
                    .expect("Could not find configurable naming.");
                let ty = typ.rust_tokens();
                let name = format_ident! { "{}", name };

                predicate_data_token_decoding.push(quote! {
                    let #name = {
                        let token = tokens.next().expect("Could not get token from decoded data");
                        let obj: #ty = #ty::from_token(token.to_owned()).expect("Could not convert token to type.");
                        obj
                    };
                });

                inputs_struct_constructor.push(quote!{
                    #name
                });
            });

            let name = predicate_inputs_name(&name);
            let ident = format_ident! { "{}", name  };
            configurable_variants.push(ident.clone());

            output = quote! {
                #output

                #[derive(Debug, Clone)]
                pub struct #ident {
                    #(#inputs_struct_fields),*
                }

                impl #ident {
                    pub fn new(data: Vec<u8>) -> Self {
                        let mut left = 0usize;
                        let decoder = ABIDecoder::default();
                        let tokens = decoder.decode_multiple(&[#(#inputs_param_types),*], &data).expect("Could not decode predicate witness data");
                        let mut tokens = tokens.iter();

                        #(#predicate_data_token_decoding)*
                        Self {
                            #(#inputs_struct_constructor),*
                        }
                    }
                }

                impl TypeId for #ident {
                    fn type_id() -> usize {
                        type_id(FUEL_TYPES_NAMESPACE, #name) as usize
                    }
                }
            }
        });

        output = quote! {
            #output

            enum PredicatesInputs {
                #(#configurable_variants(#configurable_variants)),*
            }
        };
    };

    output
}

/// Generate a set of tokens for converting predicate types between their ABI-like type
/// (e.g., `IndexerPredicate`) and their Entity-like type (e.g., `IndexerPredicateEntity`).
pub fn predicate_entity_impl_tokens() -> TokenStream {
    quote! {
        impl From<PredicateCoinOutput> for PredicateCoinOutputEntity {
            fn from(p: PredicateCoinOutput) -> Self {
                let owner = p.owner().to_owned();
                let amount = p.amount().to_owned();
                let asset_id = p.asset_id().to_owned();

                Self::new(owner, amount, asset_id)
            }
        }

        impl From<PredicateCoinOutputEntity> for PredicateCoinOutput {
            fn from(entity: PredicateCoinOutputEntity) -> Self {
                let PredicateCoinOutputEntity {
                    owner,
                    amount,
                    asset_id,
                    ..
                } = entity;

                Self::new(owner, amount, asset_id)
            }
        }

        impl From<IndexerPredicate> for IndexerPredicateEntity {
            fn from(p: IndexerPredicate) -> Self {
                let coin_output = p.coin_output();
                let spent_tx_id = p.spent_tx_id();
                let unspent_tx_id = p.unspent_tx_id();
                let configurables = p.configurables();
                let template_id = p.template_id();
                let output_index = p.output_index();
                let template_name = p.template_name();
                let bytecode = p.bytecode();

                let coin_output = PredicateCoinOutputEntity::from(coin_output.to_owned()).get_or_create();

                Self::new(
                    template_name.to_owned(),
                    configurables.to_owned(),
                    template_id.to_owned(),
                    output_index.to_owned(),
                    coin_output.id.clone(),
                    unspent_tx_id.to_owned(),
                    spent_tx_id.to_owned(),
                    bytecode.to_owned(),
                )
            }
        }

        impl From<IndexerPredicateEntity> for IndexerPredicate {
            fn from(entity: IndexerPredicateEntity) -> Self {
                let IndexerPredicateEntity {
                    configurables,
                    unspent_tx_id,
                    spent_tx_id,
                    coin_output,
                    template_id,
                    output_index,
                    template_name,
                    bytecode,
                    ..
                } = entity;

                let coin_output = PredicateCoinOutputEntity::load(coin_output).expect("Could not load coin output entity.");
                let coin_output = PredicateCoinOutput::from(coin_output);

                Self::new(
                    output_index,
                    configurables.to_vec(),
                    template_name,
                    template_id,
                    coin_output,
                    unspent_tx_id,
                    spent_tx_id,
                    bytecode,
                )
            }
        }
    }
}

/// `TokenStream` used to build args passed to indexer handlers.
pub fn dispatcher_tokens(dispatcher_name: &Ident) -> TokenStream {
    let name = dispatcher_name.to_string();
    match name.as_str() {
        // Since predicates are contained in the container `Predicates`, we pass the entire predicate container
        // to the indexer handler (rather than just a single predicate).
        "predicates_decoded" => {
            quote! { self.#dispatcher_name.clone() }
        }
        _ => {
            quote! { self.#dispatcher_name[0].clone() }
        }
    }
}

/// `TokenStream` used to build a set of decoder functions for each predicate template.
///
/// Since not every indexer will use predicates, we only generate these functions if predicates are enabled.
pub fn predicate_decoder_fn_tokens(manifest: &Manifest) -> TokenStream {
    let mut output = quote! {};
    manifest.predicates().map(|p| {
        p.templates().map(|t| {
            t.iter().for_each(|t| {
                let name = predicate_inputs_name(&t.name());
                let ident = format_ident! { "{}", name };
                let name = name.to_lowercase();
                let decoded_ident = format_ident! { "{}_decoded", name };
                let fn_name = format_ident! { "decode_{}", name };

                output = quote! {
                    #output

                    pub fn #fn_name(&mut self, data: #ident) -> anyhow::Result<()> {
                        self.#decoded_ident.push(data);
                        Ok(())
                    }
                };
            })
        })
    });

    output
}
