# Governance

## Project Leadership

Atento Core is maintained by the [We Are Progmatic](https://weareprogmatic.com) team.

## Decision Making

- **Minor changes** (bug fixes, documentation): Can be merged by any maintainer after review
- **Major changes** (new features, breaking changes): Require discussion and consensus among maintainers
- **Architecture decisions**: Documented in ADRs (Architecture Decision Records) when significant

## Maintainers

Current maintainers:
- Atento Core Team <atento@weareprogmatic.com>

## Becoming a Contributor

1. Start by contributing code, documentation, or helping in discussions
2. Consistent high-quality contributions over time
3. Demonstrate understanding of project goals and architecture
4. Community involvement and helping others

## Becoming a Maintainer

Active contributors who have demonstrated:
- Technical expertise in the codebase
- Good judgment in code reviews
- Alignment with project values
- Sustained participation over 6+ months

May be invited to become maintainers.

## Code of Conduct

All participants must follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Communication

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and discussions
- **Email**: atento@weareprogmatic.com for private matters
- **Pull Requests**: Code contributions and technical discussions

## Release Process

1. Version bump in `Cargo.toml` according to semantic versioning
2. Update `CHANGELOG.md` with notable changes
3. PR title must start with PATCH/MINOR/MAJOR
4. All CI checks must pass
5. Merge to main triggers automatic publish to crates.io

## Deprecation Policy

- Deprecated features are marked with `#[deprecated]` attribute
- Deprecation warnings remain for at least one minor version
- Deprecated features are removed only in major version updates
