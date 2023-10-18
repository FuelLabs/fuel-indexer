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

```rust
pub enum SimpleEnum {
    One: (),
    Two: (),
    Three: (),
}
```

Will be translated to GraphQL as:

```GraphQL
enum SimpleEnum {
    One
    Two
    Three
}
```

## Example

Using the `DAO-contract-abi.json`, which can be found in the `fuel-indexer` repository:

```bash
forc index new --json-abi ./packages/fuel-indexer-tests/trybuild/abi/DAO-contract-abi.json dao-indexer
```

We get the following schema:

```GraphQL
enum CreationError {
    DurationCannotBeZero
    InvalidAcceptancePercentage
}

enum InitializationError {
    CannotReinitialize
    ContractNotInitialized
}

enum ProposalError {
    InsufficientApprovals
    ProposalExecuted
    ProposalExpired
    ProposalStillActive
}

enum UserError {
    AmountCannotBeZero
    IncorrectAssetSent
    InsufficientBalance
    InvalidId
    VoteAmountCannotBeZero
}

type CallDataEntity @entity {
    id: ID!
    arguments: U64!
    function_selector: U64!
}

type CreateProposalEventEntity @entity {
    id: ID!
    proposal_info: ProposalInfoEntity!
}

type DepositEventEntity @entity {
    id: ID!
    amount: U64!
    user: Identity!
}

type ExecuteEventEntity @entity {
    id: ID!
    acceptance_percentage: U64!
    user: Identity!
}

type InitializeEventEntity @entity {
    id: ID!
    author: Identity!
    token: ContractId!
}

type ProposalEntity @entity {
    id: ID!
    amount: U64!
    asset: ContractId!
    call_data: CallDataEntity!
    gas: U64!
}

type ProposalInfoEntity @entity {
    id: ID!
    acceptance_percentage: U64!
    author: Identity!
    deadline: U64!
    executed: Boolean!
    no_votes: U64!
    proposal_transaction: ProposalEntity!
    yes_votes: U64!
}

type UnlockVotesEventEntity @entity {
    id: ID!
    user: Identity!
    vote_amount: U64!
}

type VoteEventEntity @entity {
    id: ID!
    user: Identity!
    vote_amount: U64!
}

type VotesEntity @entity {
    id: ID!
    no_votes: U64!
    yes_votes: U64!
}

type WithdrawEventEntity @entity {
    id: ID!
    amount: U64!
    user: Identity!
}
```
