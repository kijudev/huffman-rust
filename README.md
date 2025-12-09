# Huffman coding implemented in Rust

A small, self-contained implementation of Huffman coding in Rust for educational and utility purposes.

This repository contains:

- A library implementing Huffman encoding/decoding (`src/huffman.rs`).
- A simple CLI front-end that compresses and decompresses files (`src/main.rs`) using `clap`.
- Message serialization using MessagePack (`rmp-serde`) to store the Huffman tree + encoded bit stream.

The encoder uses a canonical in-memory tree representation and a `bitvec::BitVec` to store encoded bits. The CLI exposes easy-to-use `compress` and `decompress` subcommands.

## Quick build

From the crate root (`huffman-rust`), build with Cargo:

```huffman-rust/README.md#L1-10
cargo build
```

Or build a release binary:

```huffman-rust/README.md#L1-10
cargo build --release
```

## CLI usage

The CLI supports two subcommands: `compress` and `decompress`.

- `compress <INPUT> [OUTPUT]`
  Reads the file at `INPUT`, compresses it with Huffman encoding, serializes the resulting `Message` using MessagePack, and writes the serialized bytes to `OUTPUT`. If `OUTPUT` is omitted, a default filename is created by replacing the input file extension with `<ext>.huf` (for example `file.txt` -> `file.txt.huf`).

- `decompress <INPUT> [OUTPUT]`
  Reads a MessagePack-serialized Huffman message from `INPUT`, deserializes it, decodes the original bytes, and writes them to `OUTPUT`. If `OUTPUT` is omitted, a default filename is created by replacing the input file extension with `<ext>.orig` (for example `file.huf` -> `file.huf.orig`).

## Examples

Compress a file and write to an explicit output:

```huffman-rust/README.md#L1-20
cargo run -- compress path/to/input.txt path/to/output.huf
```

Compress a file and let the CLI choose the output path:

```huffman-rust/README.md#L1-20
cargo run -- compress path/to/input.txt
# -> writes to path/to/input.txt.huf (default behavior)
```

Decompress a file to a specified output file:

```huffman-rust/README.md#L1-20
cargo run -- decompress path/to/input.huf path/to/output.bin
```

Decompress and let the CLI choose the output path:

```huffman-rust/README.md#L1-20
cargo run -- decompress path/to/input.huf
# -> writes to path/to/input.huf.orig (default behavior)
```

## Notes

- Serialization format: MessagePack (via `rmp-serde`). The serialized file contains both the Huffman `Tree` (so the decoder does not need a fixed/shared tree) and the encoded bits.
- The implementation uses `bitvec` for compact bit storage; because MessagePack stores bytes, the bit container is serialized in a form compatible with `bitvec`'s serde support.
- For empty input, the encoder returns an empty message that decodes to an empty output.
- For inputs containing just one distinct symbol, the implementation assigns a non-empty code to that symbol (so repeated occurrences can be encoded as bits).
