# forc-index

A `forc` plugin for basic Fuel Indexer interaction.

## Commands

### `forc index init`

Create a new index project at the provided path. If no path is provided the current working directory will be used.

```bash
forc index init --namespace fuel
```

### `forc index new`

Create new index project at the provided path.

```bash
forc index new --namespace my_org_name
```

### `forc index start`

Start a local Fuel Indexer service.

```bash
forc index start --background
```

### `forc index deploy`

Deploy a given index project to a particular endpoint

```bash
forc index deploy --url https://index.swaysway.io --manifest my_index.manifest.yaml
```

### `forc index remove`

Kill a running indexer

```bash
forc index remove --url https://index.swayswap.io --manifest my_index.manifest.yaml
```

### `forx index revert`
Remove the current index and revert to the most recent.

```bash
forc index revert --url https://index.swayswap.io --manifest my_index.manifest.yaml
```

### `forc index check`

Check to see which indexer components you have installed.

```bash
forc index check
```

### `forc index build`

Build the index in the current directory.

```bash
forc index build --verbose
```

### `forc index postgres create`

Create a new database.

```bash
forc index postgres create postgres --persistent
```

### `forc index postgres start`

Start a previously created database.

```bash
forc index postgres start postgres
```

### `forc index postgres stop`

Stop a running database.

```bash
forc index postgres stop postgres
```

### `forc index postgres drop`

Drop a stopped database.

```bash
forc index postgres drop postgres
```
