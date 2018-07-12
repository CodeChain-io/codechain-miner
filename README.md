# CodeChain Miner [![Build Status](https://travis-ci.org/CodeChain-io/codechain-miner.svg?branch=master)](https://travis-ci.org/CodeChain-io/codechain-miner)

Mining worker for PoW algorithms in [CodeChain](https://github.com/CodeChain-io/codechain).

## Build

CodeChain miner is written in rust. We recommend setting up build environment with [rustup](https://rustup.rs/).

To build executable in release mode, run following command.
```
cargo build --release
```

Resulting binary file can be found at `target/release/codechain-miner`.

## Usage

CodeChain miner can switch between multiple mining algorithms with command line option. To run miner with specific algorithm, run:
```
codechain-miner ALGORITHM [OPTIONS]
```

### Supported Algorithms
- `blake` : [Blake2b](https://blake2.net/) with output length of 32 bytes
- `cuckoo` : [Cuckoo Cycle](https://github.com/tromp/cuckoo)

### Usage Examples
* **Blake** mining, listening to port **3334**, submitting to port **8081** :
```
codechain-miner blake -p 3334 -s 8081
```
* **Cuckoo Cycle** mining, N=16, M=8, L=6 :
```
codechain-miner cuckoo -n 16 -m 8 -l 6
```

## Configuration

### Common options

| Option | Description                    | Default | Required |
| :----: | ------------------------------ |:-------------:|:--------:|
| `-p`   | Port number to receive job     | 3333 | No |
| `-s`   | Port number to submit solution | 8080 | No |

### Blake
No configuration option available.

### Cuckoo
| Option | Description                    | Default | Required |
| :----: | ------------------------------ |:-------------:|:--------:|
| `-n`   | Number of vertices in graph | None | Yes |
| `-m`   | Number of edges in graph    | None | Yes |
| `-l`   | Length of cycle to detect   | None | Yes |
