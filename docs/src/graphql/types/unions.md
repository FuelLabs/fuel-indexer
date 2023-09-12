# Union Types

Union types are unique in that any type marked as a `union` will be converted into an Object type, who's fields are the unique set of fields over all members of the union. 

```graphql
enum TransactionLabel {
    Create
    Script
    Mint
}

type CreateTransaction @entity {
    id: ID!
    bytecode_length: UInt8!
    contract_id: ContractId!
    label: TransactionLabel!
}

type ScriptTransaction @entity {
    id: ID!
    maturity: UInt8!
    label: TransactionLabel!
}

type MintTransaction @entity {
    id: ID!
    metadata: Json
    label: TransactionLabel!
}

union Transaction = CreateTransaction | ScriptTransaction | MintTransaction
```

The `Transaction` union type above, will internally produce the following object type:

```graphql
type Transaction @entity {
    id: ID!
    bytecode_length: UInt8!
    contract_id: ContractId!
    label: TransactionLabel!
    maturity: UInt8!
    metadata: Json
}
```

> IMPORTANT: Note the order of the fields in the derived `Transaction` object type: the fields are ordered according to the unique set of fields from each of the union's members.
>
> The `id`, `bytecode_length`, `contract_id`, and `label` fields come first, from the `CreateTransaction` object type. Next comes the `maturity` field from the `ScriptTransaction` object - because the `ScriptTransaction`'s `id` and `label` fiels are already a part of the derived `Transaction` object, courtesy of the `CreateTransaction` object type. Finally, comes the `metadata` field, as part of the `MintTransaction` object type.

This `Transaction` union type from the GraphQL schema, might be used in an indexer module like so:

```rust, ignore
extern crate alloc;
use fuel_indexer_utils::prelude::*;

#[indexer(manifest = "indexer.manifest.yaml")]
mod indexer_mod {
    fn handle_event(event: Event) {
        let bytecode_length = 1024;
        let contract_id = ContractId::default();
        let label = TransactionLabel::Create;
        let maturity = 10000000;
        let metadata = None;

        let transaction = Transaction::new(bytecode_length, contract_id, label, maturity, metadata);
        transaction.save();
    }
}
```