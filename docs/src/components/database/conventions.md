# Conventions

## IDs

- IDs are often included in Sway smart contracts as a means of identifying an entity or object.
- Giving an indexable entity an ID allows you to easily find the entity after it's been indexed, using the GraphQL API included with the Fuel indexer.
- With regard to Fuel indexing specifically, the `ID` data type in your GraphQL schema maps to a `bigint primary key` in Postgres.
  - Given this, ID fields in GraphQL schema should be paired with `u64` types in a Sway contract.
- Note that a Sway contract developer can always use another data type (e.g., `str[32]`) as a type of identifier field. However, you should reference [the Fuel indexer types table](./types.md) when deciding which data types to use in certain scenarios.

> Important:
>
> 1. In the near future, we plan to support `ID`s of any type -- i.e., `Address`, `ContractId`, etc.
>
> 2. One important thing to note is that when it comes to indexing data, the developer of the Sway smart contract is responsible for creating IDs. The Fuel indexer does not use any type of auto-incrementing ID mechanism. Though as of now, `ID` types in GraphQL _do_ map to auto-incrementing `bigint`s in Postgres, we plan to phase this functionality out in the future.
>    - This is important because if a Fuel indexer operator is expecting IDs to be (for example) globally unique, then the indexer operator would have to ensure that the Sway contract generating the indexable events has some mechanism to generate those unique IDs.
