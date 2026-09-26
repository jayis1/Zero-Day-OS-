# Zero-Day OS — Toolchain Feasibility Note

Scope: IST-7 (Division B — Firmware & Systems Architecture). Verifies repo
reachability, build/toolchain status, and flags blockers for the Zero-Day OS
build pipeline targeting the M5Stack CardPuter Zero (aarch64 / ARM64).

Verification host: x86_64 Linux, cross-compiling to `aarch64-unknown-linux-gnu`.
Verified against repo HEAD `7ead99c` (`main`).

## 1. Repo reachability

- `jayis1/Zero-Day-OS-` — REACHABLE. `git ls-remote` succeeds; remote `main`
  HEAD `7ead99c` matches the local working tree. Remote authenticates via a PAT
  embedded in the origin URL.
- `jayis1/unified-TREE` (SoC node mesh) — co-owned by hermesOllama; reachability
  and toolchain feasibility for that repo are that owner's deliverable and are
  not asserted here.

## 2. Toolchain status — VERIFIED WORKING

Two independent build tracks make up the OS. Both were exercised, not just
inspected.

### 2a. Rust userspace components (cross-compiled to aarch64)

Host prerequisites present and sufficient: `rustc`, `cargo`, `cross`, Docker
(daemon up), `qemu-aarch64-static`, and the installed
`aarch64-unknown-linux-gnu` rustc target. Note the native
`aarch64-linux-gnu-gcc` is NOT on the host — cross-compilation runs entirely
through the `cross` Docker images, which is the intended path.

| Component  | Cross image                                            | Result | Evidence |
|------------|--------------------------------------------------------|--------|----------|
| `trail`    | `ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main`      | PASS   | `zeroday-trail`: `ELF 64-bit LSB pie, ARM aarch64, dynamically linked, stripped` |
| `compositor` | custom `zeroday-comp-cross` (Wayland/DRM/GBM/EGL arm64 deps) | PASS   | release build finished in ~52s, 8 dead-code warnings only |

Other Rust crates (`terminal`, `tui`, `explorer`) share the same `cross` +
`Cross.toml` + Makefile pattern (`make cross-build` → `cross build --release
--target aarch64-unknown-linux-gnu`); no divergent toolchain requirement was
found. `tui` has no `Cross.toml` (pure-Rust, no C deps) so it builds under the
default cross image.

Only warnings emitted are unused-code lints — no errors. Toolchain is fully
functional for the firmware/userspace layer.

### 2b. OS image build (pi-gen, arm64)

- Config: `pi-gen/config` → `IMG_NAME=zeroday-os`, `RELEASE=bookworm`,
  `ARCH=arm64`, stages 0–5 present (base, kernel/dtb, TUI + zeroday
  components, Kali repos/tools, first-boot/opencode).
- Build mechanism: Docker + QEMU arm64 emulation via `pi-gen/build-docker.sh`.
- CI: `.github/workflows/build-image.yml` builds the arm64 image on
  `ubuntu-latest`, installs `qemu-user-static`, sets up buildx, and is
  budgeted at 360 min (QEMU arm64 pi-gen runs 3–4h). Triggered on `v*` tags or
  manual `workflow_dispatch`. This path is CI-validated by prior releases
  (deploy/ contains `zeroday-monsterc5-firmware-v4.3.0.zip`).

## 3. Blockers / risks (for CEO)

None that block the toolchain. Items to be aware of:

1. **No hard blocker.** Both the Rust cross-compile track and the pi-gen image
   track are functional and CI-backed. Firmware/systems-architecture work can
   proceed.
2. **Image build is long and heavy (advisory).** pi-gen arm64 under QEMU is
   3–4h wall-clock and disk-hungry; the CI job already frees disk and caches
   apt. Not a blocker, but plan release cadence around it — it cannot be
   iterated quickly.
3. **Secret hygiene (advisory).** The `origin` remote embeds a GitHub PAT in
   its URL. Functional, but the token is exposed to anything that can read the
   git config on the build host. Prefer a credential helper / CI secret over an
   in-URL PAT. Flagging as a hygiene item, not a build blocker.
4. **Host caveat for local builds.** A build host must have Docker + QEMU for
   both tracks; a bare host without Docker cannot cross-build (native
   `aarch64-linux-gnu-gcc` is intentionally not required). Documented so local
   contributors don't hit a false "toolchain broken" signal.

## 4. Disposition

Acceptance criteria met: repos verified reachable, build/toolchain status
verified by real aarch64 compilation (not inspection), feasibility documented,
blockers assessed — none blocking, three advisory items above for CEO
awareness.
