# Black Hole Simulator 🕳️

An interactive black hole visualizer built with Rust + WebAssembly.
## Acknowledgements
Inspired by [kavan010's black hole lensing implementation](https://github.com/kavan010/black_hole/blob/main/2D_lensing.cpp) (C++).
Reimplemented in Rust + WebAssembly.
## Prerequisites

You'll need the following installed before running the project:

### 1. Rust

Install via [rustup](https://rustup.rs/):

Then restart your terminal.

### 2. wasm-pack

```bash
cargo install wasm-pack
```

### 3. Node.js & npm

Download from [nodejs.org](https://nodejs.org/) (LTS version recommended).

## Running the Project

To build the WebAssembly and start a local dev server:

```bash
npm install
npm start
```

This will:

1. Compile the Rust code to WebAssembly (`wasm-pack build`)
2. Start a local server at **http://localhost:9999** and open it in your browser

### Other commands

| Command         | Description                            |
| --------------- | -------------------------------------- |
| `npm run build` | Compile Rust → WASM only (no server)   |
| `npm run serve` | Start the local server only (no build) |
