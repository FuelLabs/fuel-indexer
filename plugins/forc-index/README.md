# forc-index

A `forc` plugin for basic Fuel Indexer interaction.

## Commands

### `forc index new`

Create new indexer project at the provided path.

```bash
forc index new --namespace my_org_name
```

### `forc index start`

Start a local Fuel Indexer service.

```bash
forc index start
```

### `forc index deploy`

Deploy a given indexer project to a particular endpoint

```bash
forc index deploy --url https://beta-5-indexer.fuel.network
```

### `forc index remove`

Kill a running indexer

```bash
forc index remove --url https://beta-5-indexer.fuel.network
```

### `forc index check`

Check to see which indexer components you have installed.

```bash
forc index check
```

### `forc index build`

Build the indexer in the current directory.

```bash
forc index build --verbose
```

### `forc index auth`

Authenticate against an indexer service.

```bash
forc index auth --verbose
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
