# Changelog

All notable changes to this project will be documented in this file.

## 2.0.4 - 2026-07-17
### 🐛 Bug Fixes
- fix(core): accurate throughput + GUI history trend charts
### 📚 Documentation
- docs: update README and CHANGELOG for v2.0.3
### 🔧 Chores
- chore: bump version to 2.0.4
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v2.0.3...v2.0.4
## 2.0.3 - 2026-07-17
### 🐛 Bug Fixes
- fix(core): migrate over stale sled history directory (redb open failure)
### 📚 Documentation
- docs: update README and CHANGELOG for v2.0.2
### 🔧 Chores
- chore: bump version to 2.0.3
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v2.0.2...v2.0.3
## 2.0.2 - 2026-07-17
### 📚 Documentation
- docs: update README and CHANGELOG for v2.0.1
### 📦 Other Changes
- refactor+tooling: dedup geolocation, modular nu tests, faster release
### 🔧 Chores
- chore: bump version to 2.0.2
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v2.0.1...v2.0.2
## 2.0.1 - 2026-07-17
### 🐛 Bug Fixes
- fix(gui): make run-history persistence robust and visible
### 📚 Documentation
- docs: update README and CHANGELOG for v2.0.0
### 🔧 Chores
- chore(release): v2.0.1
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v2.0.0...v2.0.1
## 2.0.0 - 2026-07-15
### ✨ Features
- feat(gui): show run history + JSON settings; add core Settings type
- feat(gui): editable settings panel (server, size, timeout, detail)
### 📚 Documentation
- docs: update README and CHANGELOG for v1.0.1
- docs: rewrite README — cleaner, fixed previews, GUI + core library docs
### 🔧 Chores
- chore: bump version to 2.0.0
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v1.0.1...v2.0.0
## 1.0.1 - 2026-07-13
### 📚 Documentation
- docs: update README and CHANGELOG for v1.0.0
- docs: update repo URLs after renaming the GitHub repo to netrunner
### 🔄 CI
- ci(release): reliable dispatchable release flow (mirror gitkraft)
- ci: install nu + git-cliff from prebuilt binaries (fast)
- ci(release): check crates.io (not local workspace) for idempotent publish
### 🔧 Chores
- chore: rework justfile for the workspace (mirror gitkraft layout)
- chore: bump version to 1.0.1
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v1.0.0...v1.0.1
## 1.0.0 - 2026-07-13
### 📚 Documentation
- docs: update README and CHANGELOG for v0.7.3
### 📦 Other Changes
- feat!: split netrunner into a Cargo workspace (core + TUI + GPUI GUI)
### 🔄 CI
- ci: make workspace CI green (GPUI on macOS, scripts workspace-aware)
### 🔧 Chores
- chore(release): v0.8.0
- chore(release): v1.0.0
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.7.3...v1.0.0
## 0.7.3 - 2026-04-05
### 📚 Documentation
- docs: update README and CHANGELOG for v0.7.1
### 📦 Other Changes
- Automate git push and remove confirmation from bump_version.nu
### 🔧 Chores
- chore: bump version to 0.7.3
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.7.1...v0.7.3
## 0.7.1 - 2026-04-05
### ➕ Added
- Add missing import for ReadableDatabase in history module
### 🔧 Chores
- chore: bump version to 0.7.1
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.7.0...v0.7.1
## 0.7.0 - 2026-04-04
### 📦 Other Changes
- Switch to redb, remove aws-lc, optimize binary size, update tests
### 🔧 Chores
- chore: bump version to 0.7.0
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.6.3...v0.7.0
## 0.6.3 - 2026-03-28
### 📚 Documentation
- docs: update README and CHANGELOG for v0.6.2
### 🔧 Chores
- chore: bump version to 0.6.3
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.6.2...v0.6.3
## 0.6.2 - 2026-03-27
### ➕ Added
- add gitea dual hosting scripts and workflows
### 📚 Documentation
- docs: update README and CHANGELOG for v0.6.1
### 🔧 Chores
- chore: bump version to 0.6.2
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.6.1...v0.6.2
## 0.6.1 - 2026-03-19
### 📦 Other Changes
- Switch tui-piechart to crates.io, update justfile, fix tests
- Use frame.area() instead of frame.size() in layout calculation
### 🔧 Chores
- chore: bump version to 0.6.1
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.6.0...v0.6.1
## 0.6.0 - 2026-03-19
### ➕ Added
- Add Gitea dual-hosting support and workflows
- Add full release workflow commands for GitHub and Gitea
- Add interactive statistics dashboard TUI with charts and history
### 🐛 Bug Fixes
- Fix sed -i usage for macOS compatibility in bump_version.sh
### 📚 Documentation
- docs: update README and CHANGELOG for v0.5.5
### 📦 Other Changes
- Remove Windows from CI and release workflow matrices
- Bump version to 0.5.7 and update release workflows
- Install git-cliff via prebuilt binary in workflows
- Adds pull recipes to justfile
### 🔄 Updated
- update gitea migration files
### 🔧 Chores
- chore: bump version to 0.5.7
- chore: bump version to 0.6.0
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.5.5...v0.6.0
## 0.5.5 - 2025-11-02
### 📚 Documentation
- docs: update README and CHANGELOG for v0.5.4
### 🔄 Updated
- Update README.md
- Update README.md
- Update README.md
### 🔧 Chores
- chore: bump version to 0.5.5
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.5.4...v0.5.5
## 0.5.4 - 2025-10-26
### 📚 Documentation
- docs: update README and CHANGELOG for v0.5.3
### 📦 Other Changes
- Remove VHS tape generator script and update demo generation
- Remove old example scripts and update VHS demos
### 🔄 Updated
- Update VHS examples to use One Dark Pro theme and new output paths
### 🔧 Chores
- chore: bump version to 0.5.4
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.5.3...v0.5.4
## 0.5.3 - 2025-10-22
### ➕ Added
- Add VHS tape generator script for examples
### 📈 Improvements
- improve spinner throbber animations, show checkmark when speed test
- Improve bandwidth monitor UI and latency reporting
### 🔄 Updated
- Update README with generate-all.sh usage instructions
### 🔧 Chores
- chore: bump version to 0.5.1
- chore: bump version to 0.5.2
- chore: bump version to 0.5.2
- chore: bump version to 0.5.3
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.5.1...v0.5.3
## 0.5.1 - 2025-10-19
### 📦 Other Changes
- Bump version to 0.5.0 and update changelog and tooling fix gitcliff
### 🔧 Chores
- chore: bump version to 0.5.0
- chore: bump version to 0.5.0
- chore: bump version to 0.5.1
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.4.0...v0.5.1
## 0.4.0 - 2025-10-19
### ✨ Features
- introduce realtime graph, simplify ui and cleanup
### ➕ Added
- Add robust geolocation and server selection system
### 📈 Improvements
- improve netrunner tui add chart for upload and download
### 🔧 Chores
- chore: bump version to 0.4.0
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.3.0...v0.4.0
## 0.3.0 - 2025-10-13
### 📚 Documentation
- docs: update README and CHANGELOG for v0.2.6
### 🔧 Chores
- chore: bump version to 0.3.0
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.2.6...v0.3.0
## 0.2.6 - 2025-10-09
### 📈 Improvements
- improve speed test algorithm, add new logo and start screen, use ratatui and shader animations, update readme
### 📦 Other Changes
- use ratatui and improve speed test quality with cloudflare
- automate rleease process, introduce just file, and extend CI
- formatting and cleanup
### 🔧 Chores
- chore: track Cargo.lock for reproducible builds
- chore: bump version to 0.2.6
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.2.5...v0.2.6
## 0.2.5 - 2025-05-31
### 📚 Documentation
- docs(readme): fix CI badge URL and update version to 0.2.5
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.2.4...v0.2.5
## 0.2.4 - 2025-05-31
### 🐛 Bug Fixes
- fix(release): update git-cliff parameters for compatibility with latest version
### 🔧 Chores
- chore(version): bump version to 0.2.4
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.2.3...v0.2.4
## 0.2.3 - 2025-05-31
### 🐛 Bug Fixes
- fix(release): update git-cliff configuration and workflow for proper changelog generation
### 🔧 Chores
- chore(version): bump version to 0.2.3
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.2.2...v0.2.3
## 0.2.2 - 2025-05-31
### 📦 Other Changes
- make netrunner speed test more robust and increase test phase for more accuracy
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.2.0...v0.2.2
## 0.2.0 - 2025-05-31
### 🐛 Bug Fixes
- fix gitcliff
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.1.8...v0.2.0
## 0.1.8 - 2025-05-31
### 📦 Other Changes
- dynamic location based test server with cloudflare fallback, simplify loading animations
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.1.7...v0.1.8
## 0.1.7 - 2025-05-30
### 📈 Improvements
- improve release workflow
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.1.6...v0.1.7
## 0.1.6 - 2025-05-30
### 📦 Other Changes
- automatic incremental versioning
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.1.5...v0.1.6
## 0.1.5 - 2025-05-30
### 📦 Other Changes
- use geolocation based internet testing, add tests and simplify CLI UI
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.1.4...v0.1.5
## 0.1.4 - 2025-05-30
### 📈 Improvements
- improve gitclif changelog generation
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.1.3...v0.1.4
## 0.1.3 - 2025-05-30
### 🐛 Bug Fixes
- fix CI & publishing tokens naming
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.1.2...v0.1.3
## 0.1.2 - 2025-05-30
### 🐛 Bug Fixes
- fix gitclif changelog issues
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.1.1...v0.1.2
## 0.1.1 - 2025-05-30
### 📦 Other Changes
- allow dirty to publish on cargo the newly generated uncommited gitclif changelog as well
**Full Changelog**: https://github.com/sorinirimies/netrunner/compare/v0.1.0...v0.1.1
## 0.1.0 - 2025-05-30
### ✅ Enhanced
- enhance the netrunner cli & setup publishing and gitclif changelog
### 📦 Other Changes
- Initial commit
- Internet speed CLI tool first draft
### 🔄 Updated
- update owner and author
