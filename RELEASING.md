# Release Process

This document outlines the process for creating new releases of Surely You Jest.

## Automated Release Process

Releases are automated using GitHub Actions. The workflow is triggered when a new tag with the format `v*` is pushed to the repository.

### Release Steps

1. Update the version in `Cargo.toml` to match the new version you want to release
2. Commit the changes:
   ```bash
   git add Cargo.toml
   git commit -m "Bump version to X.Y.Z"
   ```
3. Create and push a new tag:
   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin vX.Y.Z
   ```
4. The GitHub Actions workflow will automatically:
   - Build binaries for multiple platforms (Linux, macOS, Windows)
   - Create a GitHub release with the binaries attached
   - The release will be published on GitHub at: https://github.com/clintonmedbery/surely-you-jest/releases

### Version Numbering

We follow Semantic Versioning:
- MAJOR version for incompatible API changes
- MINOR version for new functionality in a backwards compatible manner
- PATCH version for backwards compatible bug fixes