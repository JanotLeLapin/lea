{ rust-bin
, openjdk21
, mkShell
}: mkShell {
  packages = [
    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
    openjdk21
  ];
}
