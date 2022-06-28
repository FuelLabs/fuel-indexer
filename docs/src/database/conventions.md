# Conventions

### IDs

IDs are often included in Sway smart contracts as a means of identifying an entity or object. Giving an indexable entity an ID, allows you to easily find the entity after it's been indexed, using the GraphQL API included with the Fuel Indexer.

With regard to the Fuel Indexer, the `ID` data type in GraphQL schema maps to a `bigint primary key` in Postgres. Given this, ID fields in GraphQL schema should be paired with `u64` types in a Sway contract.
- Note that a Sway contract developer can always use another data type (e.g., `str[32]`) as a type of identifier field. However, you should reference [the Fuel Indexer types table](./types.md) when deciding which data types to use in certain scenarios. 

One important thing to note is that when it comes to indexing data, the developer of the Sway smart contract is responsible for creating IDs. The Fuel Indexer does not use any type of auto-incrementing ID mechanism.
- This is important because if a Fuel Indexer operator is expecting IDs to be (for example) globally unique, then the indexer operator would have to ensure that the Sway contract generating the indexable events has some mechanism to generate those unique IDs.
