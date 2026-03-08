{
  description = "Nix flake for the Beads Task-Issue Tracker";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, nixpkgs, home-manager, ... }:
    let
      lib = nixpkgs.lib;
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = lib.genAttrs systems;
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          package =
            pkgs.stdenv.mkDerivation (finalAttrs: {
              pname = "beads-gui";
              version = "1.24.2";

              src = ./.;

              cargoRoot = "src-tauri";
              cargoDeps = pkgs.rustPlatform.importCargoLock {
                lockFile = ./src-tauri/Cargo.lock;
              };

              pnpmDeps = pkgs.fetchPnpmDeps {
                inherit (finalAttrs) pname version src;
                fetcherVersion = 3;
                hash = "sha256-2+MECDgWnpK2KG/BFJ+l3qHN8Xv4NFv0TA7v8SWik+k=";
                pnpm = pkgs.pnpm_10;
              };

              nativeBuildInputs = with pkgs; [
                cargo
                cargo-tauri
                copyDesktopItems
                makeWrapper
                nodejs_22
                pnpm_10
                pkg-config
                pkgs.pnpmConfigHook
                rustPlatform.cargoSetupHook
                rustc
                wrapGAppsHook3
              ];

              buildInputs = with pkgs; [
                glib-networking
                gtk3
                libsoup_3
                openssl
                webkitgtk_4_1
              ];

              desktopItems = [
                (pkgs.makeDesktopItem {
                  name = "beads-gui";
                  desktopName = "Beads Task-Issue Tracker";
                  comment = "Desktop interface for the Beads issue tracker";
                  exec = "beads-gui";
                  icon = "beads-gui";
                  categories = [
                    "Development"
                    "ProjectManagement"
                  ];
                  terminal = false;
                })
              ];

              buildPhase = ''
                runHook preBuild

                export HOME="$PWD/.home"
                export XDG_CACHE_HOME="$PWD/.cache"
                export CARGO_TARGET_DIR="$PWD/target"
                export CI=1
                export npm_config_cache="$PWD/.npm-cache"
                export npm_config_manage_package_manager_versions=false

                pnpm config set manage-package-manager-versions false
                pnpm generate
                cargo build --manifest-path src-tauri/Cargo.toml --release

                runHook postBuild
              '';

              installPhase = ''
                runHook preInstall

                install -Dm755 "target/release/beads-issue-tracker" \
                  "$out/bin/beads-issue-tracker"
                ln -s "$out/bin/beads-issue-tracker" "$out/bin/beads-gui"

                install -Dm644 src-tauri/icons/128x128.png \
                  "$out/share/icons/hicolor/128x128/apps/beads-gui.png"

                runHook postInstall
              '';

              passthru = {
                inherit (finalAttrs) pnpmDeps;
              };

              meta = with pkgs.lib; {
                description = "Lightweight Tauri desktop app for managing Beads issues";
                homepage = "https://github.com/w3dev33/beads-task-issue-tracker";
                license = licenses.mit;
                mainProgram = "beads-gui";
                platforms = platforms.linux;
              };
            });
        in
        {
          default = package;
          beads-gui = package;
        });

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/beads-gui";
        };
      });

      homeManagerModules.default = { config, lib, pkgs, ... }:
        let
          cfg = config.programs.beads-gui;
        in
        {
          options.programs.beads-gui = {
            enable = lib.mkEnableOption "the Beads Task-Issue Tracker desktop app";
            package = lib.mkOption {
              type = lib.types.package;
              default = self.packages.${pkgs.system}.default;
              defaultText = lib.literalExpression "inputs.beads-gui.packages.${pkgs.system}.default";
              description = "Package to install for the Beads Task-Issue Tracker.";
            };
          };

          config = lib.mkIf cfg.enable {
            home.packages = [ cfg.package ];
          };
        };
    };
}
