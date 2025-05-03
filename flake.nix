{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      config.allowUnfree = true;
    };
  in
  {
    devShells."x86_64-linux".default = pkgs.mkShell {

      packages = [
        # pkgs.clang-tools should come before pkgs.clang or else clangd can't detect headers
        # https://github.com/NixOS/nixpkgs/issues/76486
        pkgs.clang-tools
        pkgs.clang
        pkgs.gnumake
        pkgs.gdb
        pkgs.glib
        pkgs.valgrind

        pkgs.hyperfine

        # manpaths dont appear in devshells
        # https://github.com/NixOS/nixpkgs/pull/234367
        # workaround here:
        # https://discourse.nixos.org/t/how-to-get-postgres-man-pages-in-a-devshell/47321/2?u=fqidz
        (pkgs.buildEnv {
          name = "devShell";
          paths = [
            pkgs.man-pages-posix
            pkgs.man-pages
            pkgs.clang-manpages
          ];
        })
      ];
    };
  };
}
