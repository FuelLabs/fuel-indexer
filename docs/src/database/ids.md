# ID Types

There are a few important things related to the use of IDs.

> **Every GraphQL type defined in your schema file is required to have an id field.**
>
> - This field must be called `id`
> - The type of this `id` field must be a `u64`
>   - You typically want to use the `ID` type for these `id` fields
>
> **Why must every field have an ID?**
>
> Since the Fuel Indexer uses WASM runtimes to index events, a foreign function interface (FFI) is needed to call in and out of the runtime. When these calls out of the runtime are made, a pointer is passed back to the indexer service to indicate the memory location for the `id` of the type/object/entity being saved.
>
> **Is this liable to change in the future?**
>
> Yes, ideally we'd like ID's to be of _any_ type, and we plan to work towards this in the future. 👍
>