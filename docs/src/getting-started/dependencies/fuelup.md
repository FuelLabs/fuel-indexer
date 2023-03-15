# `fuelup`

We strongly recommend that you use the Fuel indexer through [`forc`, the Fuel orchestrator](https://fuellabs.github.io/sway/master/forc/index.html). You can get `forc` (and other Fuel components) by way of [`fuelup`, the Fuel toolchain manager](https://fuellabs.github.io/fuelup/latest). Install `fuelup` by running the following command, which downloads and runs the installation script.

```bash
curl \
    --proto '=https' \
    --tlsv1.2 -sSf \
    https://install.fuel.network/fuelup-init.sh | sh
```

After `fuelup` has been installed, the `forc index` command and `fuel-indexer` binaries will be available on your system.
