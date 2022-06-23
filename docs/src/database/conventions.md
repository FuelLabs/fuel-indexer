# Conventions

### IDs

IDs are often included in Sway smart contracts as a means of identifying a unique entity or object. Giving an indexable entity an ID, allows you to easily find the entity after it's been indexed, using the GraphQL API included with the Fuel Indexer.

With regard to the Fuel Indexer, the `ID` data type in GraphQL schema, maps to a `bigint primary key` in Postgres. Given this, ID fields on structs in Sway contracts should be defined as `u64` types.
- Note that a Sway contract developer can always use another data type (e.g., `str[32]`) as a type of identifier field. However, this non-u64 ID field should not be named `id`.

One important thing to note is that when it comes to indexing data, the developer of the Sway smart contract is responsible for creating IDs. The Fuel Indexer only saves schema denoted with `ID` as `bigint primary key` in the database.
- This is important because if an indexer operator is expecting `ID`s to be (for example) globally unique, then the indexer operator would have to ensure that the Sway contract generating the indexable events has some mechanism to generate those unique IDs.
