# Designing a Schema

The Fuel indexer uses GraphQL in order to allow users to query for indexed data. In this chapter, you can find information on how to leverage our supported features to efficiently get the data you want.

> ⚠️ Please note that the Fuel indexer does not support the full GraphQL specification; however, we do our best to reasonably support as much as we can.

- [Types](./types.md)
- [Scalars](./scalars.md)
- [Directives](./directives.md)
- [Relationships](./relationships.md)

## Supported Functionality

While we do our best to maintain compliance with the GraphQL specification and parity with other implementations, there are a few things that are under development or will not be implemented. Here's a table describing our GraphQL functionality:

Legend:

- 🟩 : Functionally complete
- 🟨 : Partially complete
- 🟥 : Planned but incomplete
- ⛔ : Not planned

| Functionality | Status | Notes |
|------|----------|-------|
| Arguments | 🟩 | [read the Search and Filtering section](../querying/search-and-filtering.md) |
| Aliases | 🟩 | |
| Fragments | 🟨 | inline fragments are currently not supported |
| Introspection | 🟩 | |
| GraphQL Playground | 🟩 | [read the Playground section](../querying/playground.md) |
| Pagination | 🟨 | [read the Pagination section](../querying/pagination.md) |
| Directives |🟨 | [read the Directives section](./directives.md) |
| List Types |🟨 | |
| Union Types |🟨 | |
| Federation |⛔ | |
| Variables | ⛔ | |
| Mutations | ⛔ | |
| Enums | 🟨 | |
| Interfaces | ⛔ | |
| Input Types| ⛔ | |

# Automatically generating GraphQL schema from JSON ABI

`forc index new` supports automatically generating GraphQL schema from a contract JSON ABI.

Sway `struct`s are translated into GrapQL `type`s, and the following `struct` field types are supported:

| Sway Type | GraphQL Type |
|-----------|--------------|
| u128 | U128 |
| u64 | U64 |
| u32 | U32 |
| u8 | U8 |
| i128 | I128 |
| i64 | I64 |
| i32 | I32 |
| i8 | I8 |
| bool | Boolean |
| u8[64] | Bytes64 |
| u8[32] | Bytes32 |
| u8[8] | Bytes8 |
| u8[4] | Bytes4 |
| Vec<u8>| Bytes |
| SizedAsciiString<64> | ID |
| String | String |
| str[32] | Bytes32 |
| str[64] | Bytes64 |

Sway `enum` types can also be translated. However, all enum variants must have `()` type. For example:

```
pub enum SimpleEnum {
    One: (),
    Two: (),
    Three: (),
}
```

Will be translated to GraphQL as:

```
enum SimpleEnumEntity {
    ONE
    TWO
    THREE
}
```

## Example

Using the `DAO-contract-abi.json`, which can be found in the `fuel-indexer` repository:

```
forc index new --json-abi ./packages/fuel-indexer-tests/trybuild/abi/DAO-contract-abi.json dao-indexer
```

We get the following schema:

```
enum CreationErrorEntity {
    DURATIONCANNOTBEZERO
    INVALIDACCEPTANCEPERCENTAGE
}
enum InitializationErrorEntity {
    CANNOTREINITIALIZE
    CONTRACTNOTINITIALIZED
}
enum ProposalErrorEntity {
    INSUFFICIENTAPPROVALS
    PROPOSALEXECUTED
    PROPOSALEXPIRED
    PROPOSALSTILLACTIVE
}
enum UserErrorEntity {
    AMOUNTCANNOTBEZERO
    INCORRECTASSETSENT
    INSUFFICIENTBALANCE
    INVALIDID
    VOTEAMOUNTCANNOTBEZERO
}
type CallDataEntity {
    id: ID!
    arguments: U64!
    function_selector: U64!
}
type CreateProposalEventEntity {
    id: ID!
    proposal_info: ProposalInfoEntity!
}
type DepositEventEntity {
    id: ID!
    amount: U64!
    user: Identity!
}
type ExecuteEventEntity {
    id: ID!
    acceptance_percentage: U64!
    user: Identity!
}
type InitializeEventEntity {
    id: ID!
    author: Identity!
    token: ContractId!
}
type ProposalEntity {
    id: ID!
    amount: U64!
    asset: ContractId!
    call_data: CallDataEntity!
    gas: U64!
}
type ProposalInfoEntity {
    id: ID!
    acceptance_percentage: U64!
    author: Identity!
    deadline: U64!
    executed: Boolean!
    no_votes: U64!
    proposal_transaction: ProposalEntity!
    yes_votes: U64!
}
type UnlockVotesEventEntity {
    id: ID!
    user: Identity!
    vote_amount: U64!
}
type VoteEventEntity {
    id: ID!
    user: Identity!
    vote_amount: U64!
}
type VotesEntity {
    id: ID!
    no_votes: U64!
    yes_votes: U64!
}
type WithdrawEventEntity {
    id: ID!
    amount: U64!
    user: Identity!
}
```