# Changelog

All notable changes to this project are documented here. It is generated from
[Conventional Commits](https://www.conventionalcommits.org) by
[git-cliff](https://git-cliff.org) — do not edit it by hand.

## [2.0.7](https://github.com/sorinirimies/netrunner/compare/v2.0.6...v2.0.7) — 2026-11-06

### 🔄 CI and release

- **packages:** AppImages use the downloaded appimagetool; which also matched the script's own command ([`06749aa`](https://github.com/sorinirimies/netrunner/commit/06749aaf9bfbdc6db23249f43b46187f3773094e))

## [2.0.6](https://github.com/sorinirimies/netrunner/compare/v2.0.5...v2.0.6) — 2026-11-06

### 🐛 Bug fixes

- **ci:** The nightly downgrade guard compares within a semver series ([`518f55d`](https://github.com/sorinirimies/netrunner/commit/518f55da6d149b618c6b89d8e377a81865c3a9d1))

### ♻️ Refactor

- Edition 2024, MSRV 1.90 and the shared lints; HistoryStorage::open_in instead of redirecting HOME in tests; the GUI crate builds on every system; drop unused dependencies and dead-code allows ([`6d85103`](https://github.com/sorinirimies/netrunner/commit/6d851036cee2667dc23e74327bbfe54219a6ed2c))

### 📚 Documentation

- Update README and CHANGELOG for v2.0.5 ([`3ff532b`](https://github.com/sorinirimies/netrunner/commit/3ff532b5e2f13a9a7e902ca7c2b56ed662d6d408))

### 📦 Dependencies

- Nightly cargo update 2026-11-06 ([`0c30c91`](https://github.com/sorinirimies/netrunner/commit/0c30c912b4f06d991fb617bf1527028e418d3f12))
- Hold libc at 0.2.189 on Linux with an exact requirement; nightly cargo update ([`05d3aa2`](https://github.com/sorinirimies/netrunner/commit/05d3aa21222ecafa8dd5390abd5ffe6993834870))
- List pin-utils (RUSTSEC-2026-0441, through gpui's async-std) in deny.toml ([`bb11f64`](https://github.com/sorinirimies/netrunner/commit/bb11f6462450eccae8c15d638233cf392cd59a13))
- Drop postcard's default heapless-cas feature (atomic-polyfill); list gpui's unmaintained dependencies in deny.toml ([`148ab30`](https://github.com/sorinirimies/netrunner/commit/148ab309147d12a532c6e1ee107ad4d6fb36c4d9))
- Hold libc at 0.2.189 so gpui builds on Linux; allow the NCSA and CDLA-Permissive-2.0 licences ([`60ed193`](https://github.com/sorinirimies/netrunner/commit/60ed1931a19a829ca6e3637bc27d059d25cd8f8d))

### 🔄 CI and release

- **packages:** Alien gets an absolute .deb path, so the .rpm build finds it ([`8e9eab4`](https://github.com/sorinirimies/netrunner/commit/8e9eab44cb543b779b81a9088b97dbec14fe42fc))
- Download archives to a file so curl retries a dropped transfer, instead of piping into tar ([`3259027`](https://github.com/sorinirimies/netrunner/commit/3259027c5908eb691b54127c8655ec53eb87f5ef))
- The Dependabot sweep asks for a rebase on conflicts and keeps going after a failed merge ([`c5fe0e1`](https://github.com/sorinirimies/netrunner/commit/c5fe0e1aea9ea7564f3a142fa87eb371ed13db0f))
- Merge green Dependabot PRs, majors included, with a nightly sweep; nightly commits skip ignored lockfiles ([`e331d5b`](https://github.com/sorinirimies/netrunner/commit/e331d5ba4a3ac61c1f7a5bf06ff269545c5b738b))
- Create-rust-project --desktop packaging (deb/rpm, AppImage, DMG, installer and AUR from project.nu); the GUI joins the Linux gate; tapes in Nushell; secrets scan, unused-dependency gate ([`c82f811`](https://github.com/sorinirimies/netrunner/commit/c82f811ca167a629664ae4349cefddb7781bb8ab))

## [2.0.5](https://github.com/sorinirimies/netrunner/compare/v2.0.4...v2.0.5) — 2026-11-02

### ✨ Features

- **cli:** Persistent tabbed workbench (tui-chart/spinner/rating/piechart/slider/checkbox/file-explorer); rate-limit-safe download; warnings ([`e7b3932`](https://github.com/sorinirimies/netrunner/commit/e7b393271eb3803d1d0a79212e58f9e752357003))
- **core:** Bufferbloat (loaded latency) probe, idle jitter before transfers, legacy history decode ([`f219b86`](https://github.com/sorinirimies/netrunner/commit/f219b86ae044bd1251cdcb3e764c1e64b3895821))
- **tui:** Keep final live chart until keypress, then print results ([`d0ef285`](https://github.com/sorinirimies/netrunner/commit/d0ef2855253b494732b29d998a8d3c404af14fd5))
- **tui:** Live dashboard (tui-chart, tui-spinner) and history trend view; tui-piechart 1.4 ([`1e6d6ec`](https://github.com/sorinirimies/netrunner/commit/1e6d6ec0a0327011383130d87a0f9f2057589413))
- **tui:** Theme stats TUI with tui-nuances (NETRUNNER_THEME, t/T) ([`9268d79`](https://github.com/sorinirimies/netrunner/commit/9268d79df909e2946ee2eb778d958ea2988cc6e3))

### 🐛 Bug fixes

- **core:** Fall back to OVH/thinkbroadband when Cloudflare rate-limits downloads ([`4c61350`](https://github.com/sorinirimies/netrunner/commit/4c61350bb8eccb220351c1e07fdd10a36e70ab23))
- **core:** Download endpoint 403 (25MB), sub-ms median latency, RFC3550 jitter, smooth streamed upload counting; live summary line ([`78e37b6`](https://github.com/sorinirimies/netrunner/commit/78e37b6ad87a1b65016c11300d8e4c7cca717384))

### ♻️ Refactor

- **ci:** Package macOS DMG and Windows installer with tested Nushell scripts ([`6c31443`](https://github.com/sorinirimies/netrunner/commit/6c31443ea866a78c83c4b99180d41b72882296cb))

### 📚 Documentation

- **ci:** Point NSIS header at the Nushell script ([`a93f20b`](https://github.com/sorinirimies/netrunner/commit/a93f20b89531c873eb47168fbaff4dbb8bbb9942))
- **readme:** Add the Audit and Fuzz badges ([`1b43c73`](https://github.com/sorinirimies/netrunner/commit/1b43c73a15f5df8006974775f1cb796214f37bfe))
- Fix history flag name ([`7a11731`](https://github.com/sorinirimies/netrunner/commit/7a117310860304895184fce6e550d52f977c611f))
- Update README and CHANGELOG for v2.0.4 ([`3959cb2`](https://github.com/sorinirimies/netrunner/commit/3959cb2743e1812fb2ddbbb564dc82c4cf3a1611))

### 📦 Dependencies

- Bump tui-nuances to 0.1.10 ([`5e94add`](https://github.com/sorinirimies/netrunner/commit/5e94addebe5992bbbbb53b05fc9e4a249fe04ef0))

### 🔄 CI and release

- **release:** Windows NSIS setup.exe installer ([`673dbb1`](https://github.com/sorinirimies/netrunner/commit/673dbb104cce00a080acc91a072b575c61e2382c))
- **release:** Ship macOS universal DMGnetrunnernetrunner ([`bdf0f2c`](https://github.com/sorinirimies/netrunner/commit/bdf0f2ca0e175bcac0554e6a649485e4080d230e))
- Merge Dependabot PRs only after CI succeeds (workflow_run) ([`941f5a2`](https://github.com/sorinirimies/netrunner/commit/941f5a22a2142da971ede7e921679afc204da1c3))
- Bump softprops/action-gh-release from 2 to 3 (#3) ([`fcb74c8`](https://github.com/sorinirimies/netrunner/commit/fcb74c8f450a3239aa2a14321c45b681f8b368dd))
- Bump peter-evans/create-pull-request from 6 to 8 (#2) ([`c48c432`](https://github.com/sorinirimies/netrunner/commit/c48c43219f0b07ea1ca862052d411fd7e328851d))
- Bump dependabot/fetch-metadata from 2 to 3 (#1) ([`d576792`](https://github.com/sorinirimies/netrunner/commit/d576792c4fa86379bec56f12ff3b00864501b88e))
- Auto-merge major Dependabot updates when CI passes ([`a156fba`](https://github.com/sorinirimies/netrunner/commit/a156fbaf76b970f92e439310c1454bd7b439f85d))
- **fuzz:** Pin gnu target (prebuilt cargo-fuzz defaults to musl) ([`eb7ccef`](https://github.com/sorinirimies/netrunner/commit/eb7ccefd534a01470e1b29bb8aaf696cd9e184d4))
- Fix cargo deny/audit, add fuzz targets, netrunner-web (Topcoat) dashboard, justfile + workflow upgrades ([`7f2a36f`](https://github.com/sorinirimies/netrunner/commit/7f2a36fd1e58526bf76bed3fd5e15d1285b1c256))
- Nightly merges lockfile-only updates; bump + release only when a Cargo.toml requirement changes ([`74608ed`](https://github.com/sorinirimies/netrunner/commit/74608edc034f1aba95512626ea41701a0eaa378b))
- Prebuilt Nushell, cargo-deny + cargo-audit security workflow ([`077dcac`](https://github.com/sorinirimies/netrunner/commit/077dcac2960135cb46cb385bd54ae35045144296))
- **security:** Add cargo-audit workflow and just audit recipe ([`db0025c`](https://github.com/sorinirimies/netrunner/commit/db0025c5fc7207e628af894931a2c82492600258))

### 🔧 Chores

- Ignore .codegraph index ([`51f14b7`](https://github.com/sorinirimies/netrunner/commit/51f14b75b6bbf8ea528b5e5470ffd0124fd1e267))
- **justfile:** Push/pull current branch instead of hardcoded main ([`6288e09`](https://github.com/sorinirimies/netrunner/commit/6288e096eea691cc34e418e2156e8c965fe6ea8e))
- **justfile:** Skip unconfigured remotes in *-all recipes ([`dfaf6dd`](https://github.com/sorinirimies/netrunner/commit/dfaf6dd66fe4cbdf4964dac1a252042475f7d7e7))

## [2.0.4](https://github.com/sorinirimies/netrunner/compare/v2.0.3...v2.0.4) — 2026-07-17

### 🐛 Bug fixes

- **core:** Accurate throughput + GUI history trend charts ([`802d5fa`](https://github.com/sorinirimies/netrunner/commit/802d5fa30d89d4b338804397302416a3a648e053))

### 📚 Documentation

- Update README and CHANGELOG for v2.0.3 ([`64eb7d3`](https://github.com/sorinirimies/netrunner/commit/64eb7d3f304cd84cdbbf10c7f38b57364abd26c9))

## [2.0.3](https://github.com/sorinirimies/netrunner/compare/v2.0.2...v2.0.3) — 2026-07-17

### 🐛 Bug fixes

- **core:** Migrate over stale sled history directory (redb open failure) ([`a4ef667`](https://github.com/sorinirimies/netrunner/commit/a4ef6675e0333d8fc3f1d3f377d71c3207438002))

### 📚 Documentation

- Update README and CHANGELOG for v2.0.2 ([`f7be29f`](https://github.com/sorinirimies/netrunner/commit/f7be29f55a05633bb0daac859e0beed53f2b22a9))

## [2.0.2](https://github.com/sorinirimies/netrunner/compare/v2.0.1...v2.0.2) — 2026-07-17

### ♻️ Refactor

- Dedup geolocation, modular nu tests, faster release ([`dc40c4b`](https://github.com/sorinirimies/netrunner/commit/dc40c4b7efe0cad9b3c25be29da24b614fb4096c))

### 📚 Documentation

- Update README and CHANGELOG for v2.0.1 ([`613fc37`](https://github.com/sorinirimies/netrunner/commit/613fc37bc96d91a83e7e847f3a15bd1caedc7f7b))

## [2.0.1](https://github.com/sorinirimies/netrunner/compare/v2.0.0...v2.0.1) — 2026-07-17

### 🐛 Bug fixes

- **gui:** Make run-history persistence robust and visible ([`7a2ab82`](https://github.com/sorinirimies/netrunner/commit/7a2ab8260dd03bf21591815f68f2769e100b4ffa))

### 📚 Documentation

- Update README and CHANGELOG for v2.0.0 ([`7cba86c`](https://github.com/sorinirimies/netrunner/commit/7cba86c5ce8bc08e2811e699d7ce12e955fede04))

## [2.0.0](https://github.com/sorinirimies/netrunner/compare/v1.0.1...v2.0.0) — 2026-07-15

### ✨ Features

- **gui:** Editable settings panel (server, size, timeout, detail) ([`2c9f212`](https://github.com/sorinirimies/netrunner/commit/2c9f21254e13b7a966eaa2ffc91ebb3e7dd41580))
- **gui:** Show run history + JSON settings; add core Settings type ([`0a83186`](https://github.com/sorinirimies/netrunner/commit/0a83186a969b51525d48a5acca221e01e54a1f92))

### 📚 Documentation

- Rewrite README — cleaner, fixed previews, GUI + core library docs ([`c9a82ff`](https://github.com/sorinirimies/netrunner/commit/c9a82fff03a3877a43ea07aa4d355f25972323ad))
- Update README and CHANGELOG for v1.0.1 ([`6b3a905`](https://github.com/sorinirimies/netrunner/commit/6b3a905928f3f4816f16588f71e972caf9548728))

## [1.0.1](https://github.com/sorinirimies/netrunner/compare/v1.0.0...v1.0.1) — 2026-07-13

### 📚 Documentation

- Update repo URLs after renaming the GitHub repo to netrunner ([`fc4420b`](https://github.com/sorinirimies/netrunner/commit/fc4420b49ace062071bc6061016fbc42eb22c577))
- Update README and CHANGELOG for v1.0.0 ([`0e48097`](https://github.com/sorinirimies/netrunner/commit/0e48097eb6190a612b783ce6d749f9c26d3b51e1))

### 🔄 CI and release

- **release:** Check crates.io (not local workspace) for idempotent publish ([`f7fc4f0`](https://github.com/sorinirimies/netrunner/commit/f7fc4f07db33b49529b84f1f67ded19967428ff2))
- Install nu + git-cliff from prebuilt binaries (fast) ([`dc5ef89`](https://github.com/sorinirimies/netrunner/commit/dc5ef89479732a38a6a91c6077b98ae779b39aab))
- **release:** Reliable dispatchable release flow (mirror gitkraft) ([`3c3414a`](https://github.com/sorinirimies/netrunner/commit/3c3414a243143908a82f45ac6483630716b0be2d))

### 🔧 Chores

- Rework justfile for the workspace (mirror gitkraft layout) ([`df56048`](https://github.com/sorinirimies/netrunner/commit/df5604858f732770a7d7e783f4303ab419b4b043))

## [1.0.0](https://github.com/sorinirimies/netrunner/compare/v0.7.3...v1.0.0) — 2026-07-13

### ⚠️ Breaking changes

- Split netrunner into a Cargo workspace (core + TUI + GPUI GUI) ([`edc2c2a`](https://github.com/sorinirimies/netrunner/commit/edc2c2a7544089e3069b12d5fbb8e3d11d121756))

### ✨ Features

- Split netrunner into a Cargo workspace (core + TUI + GPUI GUI) ([`edc2c2a`](https://github.com/sorinirimies/netrunner/commit/edc2c2a7544089e3069b12d5fbb8e3d11d121756))

### 📚 Documentation

- Update README and CHANGELOG for v0.7.3 ([`bde21ca`](https://github.com/sorinirimies/netrunner/commit/bde21caece253ac656d8dc955c6d437ad9f8800e))

### 🔄 CI and release

- Make workspace CI green (GPUI on macOS, scripts workspace-aware) ([`32754d0`](https://github.com/sorinirimies/netrunner/commit/32754d0bc8856f6e00d5d47e74b6cc1e49b294c1))

## [0.7.3](https://github.com/sorinirimies/netrunner/compare/v0.7.1...v0.7.3) — 2026-04-05

### 📚 Documentation

- Update README and CHANGELOG for v0.7.1 ([`b400714`](https://github.com/sorinirimies/netrunner/commit/b4007145e78cdfb4940f578290bc508f37e9d5a6))

## [0.6.3](https://github.com/sorinirimies/netrunner/compare/v0.6.2...v0.6.3) — 2026-03-28

### 📚 Documentation

- Update README and CHANGELOG for v0.6.2 ([`4b79f3b`](https://github.com/sorinirimies/netrunner/commit/4b79f3bfbf9360cd844cee7e4d5386a62225eeb8))

## [0.6.2](https://github.com/sorinirimies/netrunner/compare/v0.6.1...v0.6.2) — 2026-03-27

### 📚 Documentation

- Update README and CHANGELOG for v0.6.1 ([`42f851e`](https://github.com/sorinirimies/netrunner/commit/42f851ef40d0f94447102e0ed584363e1e17190f))

## [0.6.0](https://github.com/sorinirimies/netrunner/compare/v0.5.5...v0.6.0) — 2026-03-19

### 📚 Documentation

- Update README and CHANGELOG for v0.5.5 ([`6fb256f`](https://github.com/sorinirimies/netrunner/commit/6fb256faad77dea2d7deb36a67de871b5efc2436))

## [0.5.5](https://github.com/sorinirimies/netrunner/compare/v0.5.4...v0.5.5) — 2025-11-02

### 📚 Documentation

- Update README and CHANGELOG for v0.5.4 ([`e3dd009`](https://github.com/sorinirimies/netrunner/commit/e3dd00965a190862811e3bb926cbe4f9ec766b0d))

## [0.5.4](https://github.com/sorinirimies/netrunner/compare/v0.5.3...v0.5.4) — 2025-10-26

### 📚 Documentation

- Update README and CHANGELOG for v0.5.3 ([`69402ac`](https://github.com/sorinirimies/netrunner/commit/69402ac94f376f6b981436e4aa5c4eff9eaec98c))

## [0.3.0](https://github.com/sorinirimies/netrunner/compare/v0.2.6...v0.3.0) — 2025-10-13

### 📚 Documentation

- Update README and CHANGELOG for v0.2.6 ([`f798c52`](https://github.com/sorinirimies/netrunner/commit/f798c52d9b5b45c02d3c15613d65004b1055fd0f))

## [0.2.6](https://github.com/sorinirimies/netrunner/compare/v0.2.5...v0.2.6) — 2025-10-09

### 🔧 Chores

- Track Cargo.lock for reproducible builds ([`6de2dce`](https://github.com/sorinirimies/netrunner/commit/6de2dce9237ddede66612848a5bb0970c6431886))

## [0.2.5](https://github.com/sorinirimies/netrunner/compare/v0.2.4...v0.2.5) — 2025-05-31

### 📚 Documentation

- **readme:** Fix CI badge URL and update version to 0.2.5 ([`abb1550`](https://github.com/sorinirimies/netrunner/commit/abb155022ff4c90196d70c3d2ffa8586c0ff0fd3))

## [0.2.4](https://github.com/sorinirimies/netrunner/compare/v0.2.3...v0.2.4) — 2025-05-31

### 🐛 Bug fixes

- **release:** Update git-cliff parameters for compatibility with latest version ([`a19d2f7`](https://github.com/sorinirimies/netrunner/commit/a19d2f7165e80efc6b2c1ef9ec98ba2ad557dd02))

### 🔧 Chores

- **version:** Bump version to 0.2.4 ([`68d0ba7`](https://github.com/sorinirimies/netrunner/commit/68d0ba79c4d3eb7d436675fa2549c3f1feace902))

## [0.2.3](https://github.com/sorinirimies/netrunner/compare/v0.2.2...v0.2.3) — 2025-05-31

### 🐛 Bug fixes

- **release:** Update git-cliff configuration and workflow for proper changelog generation ([`8ed3dc8`](https://github.com/sorinirimies/netrunner/commit/8ed3dc8622e04dd592e2f13c5bb2182fb89afd91))

### 🔧 Chores

- **version:** Bump version to 0.2.3 ([`718f357`](https://github.com/sorinirimies/netrunner/commit/718f3571eb20c4b48986d96d647ac047c314b1a8))

## [0.1.0](https://github.com/sorinirimies/netrunner/releases/tag/v0.1.0) — 2025-05-30


