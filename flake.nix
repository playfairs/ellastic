{
  description = "ellastic";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, fenix, ... }@inputs:
    let
      forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;
    in
    {
      formatter = forAllSystems (
        system:
        import ./nix/formatter.nix {
          inherit inputs;
          pkgs = import nixpkgs { inherit system; };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };

          toolchain = fenix.packages.${system}.complete.toolchain;
        in
        {
          default = pkgs.mkShell {
            packages = [
              toolchain
              pkgs.pkg-config
              pkgs.ffmpeg
            ];
          };
        }
      );
    };
}
