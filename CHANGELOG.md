# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.2] - 2025-10-23

### Changed
- Updated README.md with accurate workflow examples matching actual implementation
- Simplified examples by omitting default `type: string` declarations
- Governance model changed to SQLite-inspired BDFL structure

### Added
- Working example workflows in `tests/workflows/cross-platform/`:
  - `user_greeting.yaml` - demonstrates multi-type parameters and step chaining
  - `data_pipeline.yaml` - shows data processing with validation and transformation
- New `readme_examples` executable to validate README workflows
- CLA.md: Contributor License Agreement requirement for all contributions
- Updated GOVERNANCE.md with BDFL model (Raul Rita)
- Updated CONTRIBUTING.md to reference CLA and governance model

### Fixed
- Corrected README examples to use script-based execution instead of command/args
- Fixed workflow structure documentation to use HashMap for steps (not arrays)
- Updated Quick Start code to show correct API usage

## [0.0.1] - 2025-10-22

### Added
- Initial release of atento-core
- Core workflow engine with deterministic execution
- YAML/JSON workflow definition support
- Type-safe parameter and result handling
- Step dependency resolution
- Custom executor support
- Comprehensive test suite
- Cross-platform support (Unix/Windows)

### Documentation
- API documentation
- Usage examples
- Contributing guidelines
- Code of conduct
- Security policy

[Unreleased]: https://github.com/weareprogmatic/atento-core/compare/v0.0.2...HEAD
[0.0.2]: https://github.com/weareprogmatic/atento-core/compare/v0.0.1...v0.0.2
[0.0.1]: https://github.com/weareprogmatic/atento-core/releases/tag/v0.0.1
