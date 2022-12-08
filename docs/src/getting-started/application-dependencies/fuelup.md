# `fuelup`

`fuelup` installs the Fuel toolchain from our official release channels, enabling you to easily keep the toolchain updated.

Currently, this script supports Linux/macOS systems only. For other systems, please [read the Installation chapter](https://fuellabs.github.io/fuelup/master/installation/other.html).

Installation is simple: all you need is `fuelup-init.sh`, which downloads the core Fuel binaries needed to get you started on development.

```sh
curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh
```

This will automatically install `forc`, its accompanying plugins, `fuel-core` and other key components in `~/.fuelup/bin`. Please read the [Components chapter](https://fuellabs.github.io/fuelup/master/concepts/components.html) for more info on the components installed.

The script will ask for permission to add `~/.fuelup/bin` to your `PATH`.

Otherwise, you can also pass `--no-modify-path` so that `fuelup-init` does not modify your `PATH` and will not ask for permission to do so:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh -s -- --no-modify-path
```

If you just want `fuelup` without automatically installing the `latest` toolchain, you can pass the `--skip-toolchain-installation` option:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://fuellabs.github.io/fuelup/fuelup-init.sh | sh -s -- --skip-toolchain-installation
```

For more info on how to install and use fuelup, please [read the fuelup docs](https://fuellabs.github.io/fuelup/v0.14.0/).