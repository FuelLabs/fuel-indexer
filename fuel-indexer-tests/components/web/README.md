# tests/components/web

Web components used for testing

**⚠️IF ANY CHANGES ARE MADE TO THESE BINARIES, THEY NEED TO BE REBUILT AND COPIED OVER⚠️**

```bash
./src/bin/fuel-node \
    --wallet-path $PWD/wallet.json \
    --bin-path $PWD/../../contracts/fuel-indexer/out/debug/fuel-indexer.bin &

./src/bin/web-api \
    --wallet-path $PWD/wallet.json \
    --bin-path $PWD/../../contracts/fuel-indexer/out/debug/fuel-indexer.bin &
```

### Usage

```bash
cd fuel-indexer && bash scripts/start.bash
```
