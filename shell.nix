# Optional: Use nix-shell for a reproducible development environment
# Usage: nix-shell
# This is NOT a flake, just a simple shell.nix for convenience

{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  name = "bitcoin-slices-autoresearch";

  buildInputs = with pkgs; [
    # Rust toolchain
    cargo
    rustc
    rustfmt
    clippy

    # Benchmarking and analysis
    python3
    jq
    bc

    # Optional: Profiling tools
    linuxPackages.perf
    flamegraph

    # Git (usually already available)
    git
  ];

  shellHook = ''
    echo "Bitcoin Slices Autoresearch Environment"
    echo "======================================"
    echo ""
    echo "Available commands:"
    echo "  ./test_setup.sh       - Verify setup"
    echo "  ./manual_iteration.sh - Single optimization iteration"
    echo "  ./agent_loop.sh       - Start autonomous loop"
    echo ""
    echo "Make sure your LLM server is running!"
    echo ""
  '';
}
