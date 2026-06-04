{
  description = "LP-0002 Private Multisig — Basecamp module package";

  inputs = {
    logos-nix.url = "github:logos-co/logos-nix";
    nixpkgs.follows = "logos-nix/nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
    nix-bundle-lgx = {
      url = "github:logos-co/nix-bundle-lgx";
      inputs.logos-nix.follows = "logos-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, nix-bundle-lgx, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        basecampAppSrc = pkgs.lib.cleanSourceWith {
          src = ./basecamp-app;
          filter = path: type:
            let
              rel = pkgs.lib.removePrefix ((toString ./basecamp-app) + "/") (toString path);
            in
              rel != "build" && !(pkgs.lib.hasPrefix "build/" rel);
        };

        plugin = pkgs.stdenv.mkDerivation {
          pname = "lp0002-private-multisig-basecamp";
          version = "0.1.0";
          src = basecampAppSrc;

          nativeBuildInputs = [
            pkgs.cmake
            pkgs.ninja
            pkgs.qt6.wrapQtAppsHook
          ];

          buildInputs = with pkgs.qt6; [
            qtbase
            qtdeclarative
          ];

          cmakeFlags = [ "-GNinja" ];

          installPhase = ''
            runHook preInstall
            mkdir -p "$out/lib" "$out/qml"
            cp modules/liblp0002_private_multisig.* "$out/lib/"
            if [ -f "$out/lib/liblp0002_private_multisig.dylib" ]; then
              ln -s liblp0002_private_multisig.dylib "$out/lib/lp0002_private_multisig.dylib"
            fi
            cp "$src/metadata.json" "$out/metadata.json"
            cp -r "$src/qml/." "$out/qml/"
            runHook postInstall
          '';
        };

        lgx = nix-bundle-lgx.bundlers.${system}.default plugin;
        lgxPortable = nix-bundle-lgx.bundlers.${system}.portable plugin;
      in {
        packages = {
          default = plugin;
          plugin = plugin;
          lgx = lgx;
          lgx-portable = lgxPortable;
        };
      });
}
