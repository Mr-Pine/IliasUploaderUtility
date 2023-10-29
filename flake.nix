{
  inputs = {
    # Needs rustc >= 1.70.0
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, naersk }:
    let forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;
    in
    rec {
      packages = flake-packages;
      flake-packages = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          naersk' = pkgs.callPackage naersk { };
        in
        rec {
          ilias-uploader = naersk'.buildPackage
            {
              root = ./.;
              nativeBuildInputs = with pkgs; [ pkg-config openssl ];
            };
          default = ilias-uploader;
        }
      );
    };
}
