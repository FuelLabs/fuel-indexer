# Index Registration

- The Fuel indexer service allows users to upload new indices at runtime, with absolutely no downtime required to start using your new index.
- Meaning, as soon as users upload new assets to the service, those assets are immediately registered, and a new executor is created using the new assets. 
  - This provides the benefit of no service downtime, and allows users to immediately get started using their new index.

## Usage

We encourage users to use the [`forc index`](./../../plugins/forc-index.md) plugin for most (if not all) Fuel Indexer interaction.

### With `forc index`

To upload assets using `forc index` you'd simply use

```bash
forc index deploy --manifest my_index.manifest.yaml --url http://127.0.0.1:29987
```

### With `curL`

- An example of registering a new index via the command line:

```bash
curl -v http://127.0.0.1:29987/api/index/fuel_indexer_test/index1 \
    -F "manifest=@my_index_manifest.yaml" \
    -F "wasm=@my_index_module.wasm" \
    -F "schema=@my_index_schema.graphql" \
    -H 'Content-type: multipart/form-data' -H "Authorization: foo" | json_pp
```

> In the example upload request above:
>
> - `fuel_indexer_test` is the name of our `namespace`
> - `index1` is the `identifier` of our index
