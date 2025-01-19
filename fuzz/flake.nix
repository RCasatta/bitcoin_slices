{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [
          # Rust nightly toolchain
          (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default))

          # Required for cargo-fuzz
          cargo-fuzz

          # Common build dependencies
          pkg-config
        ];

        # Set RUST_BACKTRACE for better error reporting
        RUST_BACKTRACE = 1;
      };
    };
}
