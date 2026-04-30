{
  description = "Google Chat - Tauri wrapper for chat.google.com";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # Use a recent stable Rust toolchain
        rustToolchain = pkgs.rust-bin.stable.latest.default;

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        # Native build inputs required for Tauri/GTK/WebKit on Linux
        nativeBuildInputs = with pkgs; [
          pkg-config
          rustToolchain
          wrapGAppsHook3
        ];

        buildInputs = with pkgs; [
          openssl
          webkitgtk_4_1
          gtk3
          libsoup_3
          glib-networking
          gst_all_1.gstreamer
          gst_all_1.gst-plugins-base
          gst_all_1.gst-plugins-good
          librsvg
          libappindicator-gtk3
        ];

        # Filter source to include Rust files plus Tauri-specific assets
        # (icons, config, capabilities, frontend) that are needed at build time
        tauriSrc = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter =
            path: type:
            let
              baseName = builtins.baseNameOf path;
              relPath = pkgs.lib.removePrefix (toString ./. + "/") (toString path);
            in
            # Always include these Tauri-specific directories/files
            (pkgs.lib.hasPrefix "icons" relPath)
            || (pkgs.lib.hasPrefix "capabilities" relPath)
            || (pkgs.lib.hasPrefix "src-frontend" relPath)
            || (baseName == "tauri.conf.json")
            ||
              # Non-Rust assets in src/ embedded via include_str!/include_bytes!
              # at compile time (.js for notification script, .ttf for badge font)
              (pkgs.lib.hasPrefix "src/" relPath && (pkgs.lib.hasSuffix ".js" baseName || pkgs.lib.hasSuffix ".ttf" baseName))
            ||
              # Include standard Rust/Cargo files
              (craneLib.filterCargoSources path type);
        };

        # Common arguments shared between deps-only and final builds
        commonArgs = {
          src = tauriSrc;
          strictDeps = true;

          inherit nativeBuildInputs buildInputs;
        };

        # Build only the cargo dependencies (for caching)
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the full package
        google-chat-wrapper = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;

            # wrapGAppsHook3 automatically wraps the binary with GTK/GLib
            # runtime env vars (GDK_PIXBUF_MODULE_FILE, GIO_MODULE_DIR,
            # XDG_DATA_DIRS, GSETTINGS_SCHEMA_DIR, etc.).
            # We add libappindicator (loaded via dlopen) and force Wayland.
            preFixup = ''
              gappsWrapperArgs+=(
                --prefix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath [ pkgs.libappindicator-gtk3 ]}
                --set GDK_BACKEND wayland
              )
            '';
          }
        );

      in
      {
        checks = {
          inherit google-chat-wrapper;
        };

        packages = {
          default = google-chat-wrapper;
          google-chat-wrapper = google-chat-wrapper;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = google-chat-wrapper;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = with pkgs; [
            # Additional dev tools
            rust-analyzer
          ];

          # Runtime env vars for the dev shell
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
        };
      }
    );
}
