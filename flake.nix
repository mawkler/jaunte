{
  description = "Run-or-raise for Hyprland";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    naersk.url = "github:nix-community/naersk/master";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      naersk,
      ...
    }:
    let
      systems = [
        "x86_64-linux"
        "x86_64-darwin"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;

      # The daemon runs on GLib's main loop.
      buildInputs = pkgs: [ pkgs.glib ];
    in
    {
      packages = forAllSystems (system: {
        default =
          let
            pkgs = import nixpkgs { inherit system; };
            naersk-lib = pkgs.callPackage naersk { };
          in
          naersk-lib.buildPackage {
            src = ./.;
            meta.mainProgram = "jaunte";
            meta.license = pkgs.lib.licenses.gpl3Plus;
            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = buildInputs pkgs;
          };
      });

      nixosModules = {
        jaunte = import ./nix/nixos-module.nix self;
        default = self.nixosModules.jaunte;
      };

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/jaunte";
        };
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs =
              (with pkgs; [
                cargo
                rustc
                rustfmt
                rust-analyzer
                pre-commit
                rustPackages.clippy
                # Draws the switcher for a daemon started with `cargo run`.
                # It can only load the system's graphics driver while this
                # nixpkgs is at least as new as the system's, so keep the lock
                # up to date.
                quickshell
              ])
              ++ buildInputs pkgs;
            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          };
        }
      );
    };
}
