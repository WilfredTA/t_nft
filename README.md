# Trampoline NFT (tNFT)

A simple, extensible standard for NFTs over Cells (Generalized UTXOs).

Read more about tNFT [here](https://mirror.xyz/tempestlabs.eth).

Tests are written using the Trampoline SDK. Trampoline provides transaction generation capabilities via "pipelined" smart contracts for better composability and easier development of business logic.
# Building and Running
Build contracts:

``` sh
capsule build
```

Run tests:

``` sh
capsule test
```

# Extensible Logic
Possible designs:
1. For each input at idx i, witness at idx i has args.input_type == argv passed to extension script.. outputs?
2. No argv - just plain exec & the executed script decides where to read & whether parameterized by witness args
3. ScriptArg == (script_hash, Option<ArgStruct>, ...) & if arg struct is not None, pass to script 