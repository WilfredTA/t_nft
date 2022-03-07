# This script is compatible with Capsule.
#

# Before you run this script, make sure the following are done:
# 1. install ckb-binary-patcher
#    cargo install --git https://github.com/xxuejie/ckb-binary-patcher.git
# 2. add target
#    rustup  target add riscv64imac-unknown-none-elf

set -e
BIN=./target/riscv64imac-unknown-none-elf/debug/trampoline-nft

cd "$(dirname "${BASH_SOURCE[0]}")"
mkdir -p build/debug

cd contracts/trampoline-nft

echo "cargo build the contract ..."
cargo build --target riscv64imac-unknown-none-elf
cd ../../

echo "patch the binary ..."
ckb-binary-patcher -i ${BIN} -o ${BIN}

echo "copy the binary to build folder ..."
cp ${BIN} ./build/debug/trampoline-nft

echo "build contract done!"
echo "try 'cargo test -p tests' to run tests."