VERSION --global-cache 0.7

# earthly's `rust`-lib, provides nice automatic caching for rust-builds, see https://docs.earthly.dev/featured-guides/rust
IMPORT github.com/earthly/lib/rust:2.2.11 AS rust

alpine-with-nix:
  FROM alpine:edge
  # need the 'testing'-repo to install `nix`
  RUN echo "http://dl-cdn.alpinelinux.org/alpine/edge/testing" >> /etc/apk/repositories
  RUN apk add --no-cache nix tini bash
  RUN mkdir -p /etc/nix && echo "extra-experimental-features = nix-command flakes" >> /etc/nix/nix.conf
  # use a sane entrypoint, i.e., something that's PID 1-compatible
  ENTRYPOINT ["/sbin/tini", "--"]
  CMD ["bash"]
  # need to explicitly delete `/bin/sh` first, because it's a symlink to `/bin/busybox`, and `COPY` would actually follow the symlink and replace `/bin/busybox` instead.
  RUN rm /bin/sh
  # copy in our own `sh`, which wraps `bash`, and which sources `/root/sh_env` for every `RUN`-command.
  # we use this to execute all `RUN`-commands in our flake env :)
  COPY support/ci_sh.sh /bin/sh
  SAVE IMAGE ultra-game:alpine-with-nix

nix-dev-shell:
  FROM +alpine-with-nix
  WORKDIR /earthly-build
  COPY flake.nix flake.lock rust-toolchain.toml .
  # cache `/nix`, especially `/nix/store`, with correct chmod and a global id, so we can reuse it
  CACHE --chmod 0755 --id nix-store /nix
  # build our dev-shell, creating a gcroot, so it won't be garbage collected by nix.
  # TODO: `x86_64-linux` is hardcoded here, but it would be nice to determine it dynamically.
  RUN nix build --out-link /root/flake-devShell-gcroot .#devShells.x86_64-linux.default
  # set up our `/root/sh_env` file to print some info and source our flake env, will be used by ALL `RUN`-commands!
  # RUN echo 'echo Running in flake environment!' > /root/sh_env
  # RUN echo 'pstree -p' >> /root/sh_env
  RUN nix print-dev-env >> /root/sh_env
  # earthly's `rust`-lib uses `RUN`-commands, to execute `cargo` etc., and these will now also run in the flake env :)
  DO rust+INIT --keep_fingerprints=true
  SAVE IMAGE ultra-game:nix-dev-shell

build:
  FROM +nix-dev-shell
  # build with earthly's `rust`-lib, according to https://docs.earthly.dev/featured-guides/rust
  COPY --keep-ts Cargo.toml Cargo.lock ./
  COPY --keep-ts --dir src  ./
  COPY --keep-ts --dir .cargo  ./
  DO rust+CARGO --args="build --release" --output="release/[^/\.]+"
  SAVE IMAGE ultra-game:latest
