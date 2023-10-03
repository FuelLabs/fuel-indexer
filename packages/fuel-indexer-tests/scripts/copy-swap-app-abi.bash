#!/bin/bash

# This script copies the JSON ABI for each Sway application into the indexer's
# trybuild tests.
#
# This helps ensure that the indexers can be built with Sway projects that more
# closely resemble "real-world" contracts, as oppposed to just indexer test
# contracts.
#
# This script should be run from the repository root, and might have to be updated
# to account for the addition/removal of Sway applications.

usage() {
  echo "Usage: $0 [options] <directory_root_path>"
  echo "Options:"
  echo "  -h, --help       Show this help message and exit."
  echo
  echo "Arguments:"
  echo "  <directory_root_path>  The root path of the sway-application repository."
  echo
}

swayapps_root=$1

if [[ "$1" == "-h" || "$1" == "--help" ]]; then
  usage
  exit 0
fi

if [[ -z "$1" || ! -d "$1" ]]; then
  echo "Error: Invalid or missing directory root path."
  usage
  exit 1
fi

testdir=$(realpath $(dirname $(dirname $0)))
abidir=$testdir/trybuild/abi
echo $abidir

paths=(
    "AMM/project/contracts/AMM-contract"
    "AMM/project/contracts/exchange-contract"
    "DAO/project/contracts/DAO-contract"
    "OTC-swap-predicate/project/predicates/swap-predicate"
    "airdrop/project/contracts/asset-contract"
    "airdrop/project/contracts/distributor-contract"
    "multisig-wallet/project/contracts/multisig-contract"
    "escrow/project/contracts/escrow-contract"
    "timelock/project/contracts/timelock-contract"
    "auctions/english-auction/project/contracts/auction-contract"
    "name-registry/project/contracts/registry-contract"
    "oracle/project/contracts/oracle-contract"
)

for path in "${paths[@]}"; do
    cd $swayapps_root/$path
    forc build
    cp -fv $path/out/debug/*-abi.json $abidir/
done

