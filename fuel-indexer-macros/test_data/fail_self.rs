use fuel_indexer_macros::indexer;

#[indexer(
    abi = r#"
        [
          {
            "type": "function",
            "inputs": [
              {
                "name": "num",
                "type": "u64",
                "components": null,
                "typeArguments": null
              }
            ],
            "name": "gimme_someevent",
            "outputs": [
              {
                "name": "",
                "type": "struct SomeEvent",
                "components": [
                  {
                    "name": "id",
                    "type": "u64",
                    "components": null,
                    "typeArguments": null
                  },
                  {
                    "name": "account",
                    "type": "b256",
                    "components": null,
                    "typeArguments": null
                  }
                ],
                "typeArguments": null
              }
            ]
          },
          {
            "type": "function",
            "inputs": [
              {
                "name": "num",
                "type": "u64",
                "components": null,
                "typeArguments": null
              }
            ],
            "name": "gimme_anotherevent",
            "outputs": [
              {
                "name": "",
                "type": "struct AnotherEvent",
                "components": [
                  {
                    "name": "id",
                    "type": "u64",
                    "components": null,
                    "typeArguments": null
                  },
                  {
                    "name": "account",
                    "type": "b256",
                    "components": null,
                    "typeArguments": null
                  },
                  {
                    "name": "hash",
                    "type": "b256",
                    "components": null,
                    "typeArguments": null
                  }
                ],
                "typeArguments": null
              }
            ]
          }
        ]
    "#,
    namespace = "test_namespace",
    schema = r#"
        schema {
            query: QueryRoot
        }

        type QueryRoot {
            thing1: Thing1
            thing2: Thing2
        }

        type Thing1 {
            id: ID!
            account: Address!
        }


        type Thing2 {
            id: ID!
            account: Address! @indexed
            hash: Bytes32! @indexed
        }
    "#,
)]
mod indexer {
    fn function_one(self, event: SomeEvent) {
        let SomeEvent { id, account } = event;

        let t1 = Thing1 { id, account };
        t1.save();
    }
}


