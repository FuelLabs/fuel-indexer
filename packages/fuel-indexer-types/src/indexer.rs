use crate::{
    fuel::{Output, Witness},
    scalar::{Address, AssetId, Bytes32, ID},
    type_id, TypeId, BETA4_CHAIN_ID, FUEL_TYPES_NAMESPACE,
};
use fuels::accounts::predicate::Predicate as SDKPredicate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait GraphQLEntity {
    /// Return the GraphQL schema fragment for the entity.
    fn schema_fragment() -> &'static str;
}

/// Native GraphQL `TypeDefinition` used to keep track of chain metadata.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexMetadata {
    /// Metadata identifier.
    pub id: ID,

    /// Time of metadata.
    pub time: u64,

    /// Block height of metadata.
    pub block_height: u32,

    /// Block ID of metadata.
    pub block_id: String,
}

impl GraphQLEntity for IndexMetadata {
    /// Return the GraphQL schema fragment for the `IndexMetadata` type.
    fn schema_fragment() -> &'static str {
        r#"

type IndexMetadataEntity @entity {
    id: ID!
    time: U64!
    block_height: U32!
    block_id: Bytes32!
}
"#
    }
}

impl TypeId for IndexMetadata {
    /// Return the type ID for `IndexMetadata`.
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "IndexMetadata") as usize
    }
}

/// Native GraphQL `TypeDefinition` used to keep track of chain metadata.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PredicateCoinOutput {
    /// Owner of the predicate.
    owner: Address,

    /// Amount of the predicate.
    amount: u64,

    /// Asset ID of the predicate.
    asset_id: AssetId,
}

impl PredicateCoinOutput {
    /// Create a new `PredicateCoinOutput`.
    pub fn new(owner: Address, amount: u64, asset_id: AssetId) -> Self {
        Self {
            owner,
            amount,
            asset_id,
        }
    }

    /// Get the owner of the UTXO.
    pub fn owner(&self) -> &Address {
        &self.owner
    }

    /// Get the amount of the UTXO.
    pub fn amount(&self) -> u64 {
        self.amount
    }

    /// Get the asset ID of the UTXO.
    pub fn asset_id(&self) -> &AssetId {
        &self.asset_id
    }
}

impl GraphQLEntity for PredicateCoinOutput {
    /// Return the GraphQL schema fragment for the `PredicateCoinOutput` type.
    fn schema_fragment() -> &'static str {
        r#"

type PredicateCoinOutputEntity @entity {
    id: ID!
    owner: Address!
    amount: U64!
    asset_id: AssetId!
}
"#
    }
}

impl TypeId for PredicateCoinOutput {
    /// Return the type ID for `PredicateCoinOutput`.
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "PredicateCoinOutput") as usize
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PredicateWitnessData {
    /// Hash of associated predicate bytecode
    template_id: Bytes32,

    /// Configurable constants in predicates.
    output_index: u64,

    /// Configurables injected into the predicate.
    configurables: Vec<u8>,

    /// Namespace in which predicate lives.
    template_name: String,

    /// Predicate bytecode.
    bytecode: Vec<u8>,
}

impl PredicateWitnessData {
    /// Return the template ID of this associated predicate.
    pub fn template_id(&self) -> &Bytes32 {
        &self.template_id
    }

    /// Return the output index of the UTXO associated with this predicate.
    pub fn output_index(&self) -> u64 {
        self.output_index
    }

    /// Return the configuration schema of the associated predicate.
    pub fn configurables(&self) -> &Vec<u8> {
        &self.configurables
    }

    /// Return the template_name of the associated predicate.
    pub fn template_name(&self) -> &String {
        &self.template_name
    }

    /// Return the bytecode of the associated predicate.
    pub fn bytecode(&self) -> &Vec<u8> {
        &self.bytecode
    }
}
impl TryFrom<Witness> for PredicateWitnessData {
    type Error = bincode::Error;
    /// Convert from `Witness` to `PredicateWitnessData`.
    fn try_from(witness: Witness) -> Result<Self, Self::Error> {
        let data: Vec<u8> = witness.into_inner();
        bincode::deserialize(&data)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexerPredicate {
    /// Hash of associated predicate bytecode.
    template_id: Bytes32,

    /// Namespace in which predicate lives.
    template_name: String,

    /// configurables to the predicate.
    configurables: Vec<u8>,

    /// Relevant TX output indices of predicates that need to be watched.
    output_index: u64,

    /// UTXO held by predicate.
    coin_output: PredicateCoinOutput,

    /// ID of transaction in which predicate was created.
    unspent_tx_id: Bytes32,

    /// ID of transaction in which predicate was spent.
    spent_tx_id: Option<Bytes32>,

    /// Bytecode of the predicate.
    bytecode: Vec<u8>,
}

impl IndexerPredicate {
    /// Create a new `IndexerPredicate`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        output_index: u64,
        configurables: Vec<u8>,
        template_name: String,
        template_id: Bytes32,
        coin_output: PredicateCoinOutput,
        unspent_tx_id: Bytes32,
        spent_tx_id: Option<Bytes32>,
        bytecode: Vec<u8>,
    ) -> Self {
        Self {
            template_id,
            configurables,
            template_name,
            output_index,
            coin_output,
            unspent_tx_id,
            spent_tx_id,
            bytecode,
        }
    }

    /// Get the predicate bytecode hash.
    pub fn template_id(&self) -> &Bytes32 {
        &self.template_id
    }

    /// Get the configuration schema of the predicate.
    pub fn configurables(&self) -> &Vec<u8> {
        &self.configurables
    }

    /// Get the predicate data output index.
    pub fn output_index(&self) -> u64 {
        self.output_index
    }

    /// Get the UTXO held by the predicate.
    pub fn coin_output(&self) -> &PredicateCoinOutput {
        &self.coin_output
    }

    /// Get the output transaction ID of the predicate.
    pub fn unspent_tx_id(&self) -> &Bytes32 {
        &self.unspent_tx_id
    }

    /// Get the output transaction ID of the predicate.
    pub fn spent_tx_id(&self) -> &Option<Bytes32> {
        &self.spent_tx_id
    }

    /// Get the template_name of the predicate.
    pub fn template_name(&self) -> &String {
        &self.template_name
    }

    /// Get the bytecode of the predicate.
    pub fn bytecode(&self) -> &Vec<u8> {
        &self.bytecode
    }

    /// Create a new `IndexerPredicate` from a `PredicateWitnessData` and the corresponding
    /// UTXO associated with that `Witness` data.
    pub fn from_witness(
        data: PredicateWitnessData,
        tx_id: Bytes32,
        output: Output,
    ) -> Self {
        let coin_output = match output {
            Output::CoinOutput(coin_output) => PredicateCoinOutput::new(
                coin_output.to,
                coin_output.amount,
                coin_output.asset_id,
            ),
            // FIXME: What to do here?
            _ => todo!(),
        };

        let PredicateWitnessData {
            template_id,
            output_index,
            configurables,
            template_name,
            bytecode,
        } = data;

        Self {
            template_id,
            configurables,
            output_index,
            template_name,
            coin_output,
            unspent_tx_id: tx_id,
            spent_tx_id: None,
            bytecode,
        }
    }
}

impl TypeId for IndexerPredicate {
    /// Return the type ID for `IndexerPredicate`.
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "IndexerPredicate") as usize
    }
}

impl GraphQLEntity for IndexerPredicate {
    /// Return the GraphQL schema fragment for the `IndexerPredicate` type.
    ///
    /// The structure of this fragment should always match `fuel_indexer_types::Predicate`.
    fn schema_fragment() -> &'static str {
        r#"

type IndexerPredicateEntity @entity {
    id: ID!
    template_name: String!
    configurables: Bytes!
    template_id: Bytes32!
    output_index: U64!
    coin_output: PredicateCoinOutputEntity!
    unspent_tx_id: Bytes32!
    spent_tx_id: Bytes32
    bytecode: Bytes!
}"#
    }
}

/// The indexer's public representation of a predicate.
///
/// This is a manual copy of `fuels::accounts::predicate::Predicate` due to the
/// fact that `fuels::accounts::predicate::Predicate` is not serializable.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Predicate {
    /// Address of the predicate.
    ///
    /// Using `Address` because `Bech32Address` is not serializable.
    address: Address,

    /// Bytecode of the predicate.
    code: Vec<u8>,

    /// configurables injected into the predicate.
    ///
    /// Using `Vec<u8> because `UnresolvedBytes` is not serializable.
    data: Vec<u8>,

    /// Chain ID of the predicate.
    chain_id: u64,
}

impl From<SDKPredicate> for Predicate {
    /// Convert from `SDKPredicate` to `Predicate`.
    fn from(predicate: SDKPredicate) -> Self {
        Self {
            address: predicate.address().into(),
            code: predicate.code().to_vec(),
            data: predicate.data().clone().resolve(0),
            chain_id: BETA4_CHAIN_ID,
        }
    }
}

impl Predicate {
    /// Address of the predicate.
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Bytecode of the predicate.
    pub fn code(&self) -> &[u8] {
        &self.code
    }

    /// configurables injected into the predicate.
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Chain ID of the predicate.
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }
}

impl TypeId for Predicate {
    /// Return the type ID for `Predicate`.
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "Predicate") as usize
    }
}

/// Container of all predicates extracted from a block for a given indexer.
#[derive(Default, Debug, Clone)]
pub struct Predicates {
    /// List of predicates.
    items: HashMap<String, IndexerPredicate>,
}

impl Predicates {
    /// Create a new `Predicates` instance.
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }

    /// Add a predicate to the index.
    pub fn add(&mut self, id: String, predicate: IndexerPredicate) {
        self.items.insert(id, predicate);
    }

    /// Get a predicate by ID.
    pub fn get(&self, id: &str) -> Option<&IndexerPredicate> {
        self.items.get(id)
    }

    /// Get the number of predicates in the index.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl TypeId for Predicates {
    /// Return the type ID for `Predicates`.
    fn type_id() -> usize {
        type_id(FUEL_TYPES_NAMESPACE, "Predicates") as usize
    }
}
