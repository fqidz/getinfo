{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
  let
    pkgs = import nixpkgs {
      system = "x86_64-linux";
      # config.allowUnfree = true;
    };
  in
  {
    devShells."x86_64-linux".default = pkgs.mkShell {
      packages = [
        pkgs.rustc
        pkgs.rust-analyzer
        pkgs.rustfmt
        pkgs.clippy
        pkgs.cargo
        pkgs.cargo-workspaces
        pkgs.cargo-expand

        pkgs.hyperfine
        pkgs.upx
      ];

      buildInputs = [
        pkgs.lld
      ];
    };
  };
}
