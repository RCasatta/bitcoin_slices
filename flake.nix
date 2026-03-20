{
  description = "Bitcoin slices - zero allocation Bitcoin parsing library";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        # Read the rust-toolchain.toml to get the exact version
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rustToolchain
          ];

          # Optionally add additional development tools
          nativeBuildInputs = with pkgs; [
            # For criterion benchmarks
            gnuplot
          ];

          # Use a separate cargo home to avoid conflicts with system cargo cache
          # This prevents issues when the system has a newer Rust version
          shellHook = ''
            export CARGO_HOME="$PWD/.cargo-nix"
          '';
        };
      }
    );
}
