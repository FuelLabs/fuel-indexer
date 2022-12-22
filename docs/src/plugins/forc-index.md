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
forc index new --namespace fuel --path /home/fuel/projects
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

Stop and remove a running index

```bash
forc index remove --url https://index.swayswap.io --manifest my_index.manifest.yaml
```

### `forc index check`

Check to see which indexer components you have installed

```bash
forc index check
```
