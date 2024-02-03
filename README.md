# Lea

The compiler implemented in Rust for my toy language (Lea), targeting the JVM.

```ts
module Main;

fn main(args: String[]) {
  print("Hello, World!");
}
```

## Usage

Download and compile the crate:

```sh
git clone https://github.com/JanotLeLapin/lea
cd lea
cargo build --release
```

Compile and run your program:

```sh
./target/release/leac myleafile.lea
java Main # prints: Hello, World!
```

## Tests

If you have Nix installed on your system, you may run the [unit tests](./test) with `nix run .#tests`.
