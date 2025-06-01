# TribeChain GitHub Actions CI/CD

This directory contains the GitHub Actions workflows for the TribeChain project, providing comprehensive continuous integration and deployment capabilities.

## Workflows

### 🔧 Main CI Pipeline (`ci.yml`)

The primary CI workflow that runs on every push and pull request to `main` and `develop` branches.

**Features:**
- **Multi-platform testing**: Ubuntu, Windows, macOS
- **Multiple Rust versions**: Stable and beta
- **Code quality checks**: Formatting, linting with Clippy
- **Security auditing**: Dependency vulnerability scanning
- **Code coverage**: Coverage reports with Codecov integration
- **Binary builds**: Cross-platform release binaries
- **Docker images**: Multi-architecture container builds
- **Performance benchmarks**: Automated benchmark tracking
- **Automated releases**: Tag-based release creation

**Artifacts:**
- Release binaries for Linux, Windows, macOS (x64 and ARM64)
- Docker images pushed to Docker Hub
- Coverage reports
- Benchmark results

### 📱 ESP32 Build Pipeline (`esp32.yml`)

Specialized workflow for ESP32/ESP8266 microcontroller builds.

**Features:**
- **Multi-target support**: ESP32, ESP32-S2, ESP32-S3, ESP32-C3, ESP8266
- **PlatformIO integration**: Arduino framework builds
- **Firmware validation**: Build verification for all targets
- **Example generation**: Creates complete ESP32 mining examples
- **Documentation**: Generates ESP32-specific documentation

**Artifacts:**
- Compiled firmware for all ESP32 variants
- ESP32 documentation and examples

### 📚 Documentation Pipeline (`docs.yml`)

Automated documentation generation and deployment.

**Features:**
- **Rust API docs**: Generated from source code comments
- **User guide**: mdBook-based comprehensive documentation
- **GitHub Pages deployment**: Automatic publishing
- **Link validation**: Checks for broken links in markdown files
- **Multi-format output**: HTML documentation with search

**Outputs:**
- API documentation at `/api/`
- User guide at `/book/`
- Combined documentation portal

### 🔄 Dependency Management (`dependabot.yml`)

Automated dependency updates using GitHub's Dependabot.

**Configuration:**
- **Weekly updates**: Every Monday at 9:00 AM
- **Workspace-aware**: Separate configs for each Cargo workspace
- **Grouped updates**: Minor and patch updates grouped together
- **Auto-labeling**: Automatic PR labeling and assignment
- **GitHub Actions updates**: Keeps workflow actions current

## Setup Requirements

### Repository Secrets

For full functionality, configure these repository secrets:

```bash
# Docker Hub (for container publishing)
DOCKER_USERNAME=your_dockerhub_username
DOCKER_PASSWORD=your_dockerhub_token

# Codecov (for coverage reporting)
CODECOV_TOKEN=your_codecov_token
```

### GitHub Pages

Enable GitHub Pages in repository settings:
1. Go to Settings → Pages
2. Set Source to "GitHub Actions"
3. Documentation will be available at `https://username.github.io/repository-name/`

### Branch Protection

Recommended branch protection rules for `main`:

```yaml
required_status_checks:
  strict: true
  contexts:
    - "Test Suite (ubuntu-latest, stable)"
    - "Test Suite (windows-latest, stable)"
    - "Test Suite (macos-latest, stable)"
    - "Security Audit"
    - "ESP32 Build Validation"

enforce_admins: true
required_pull_request_reviews:
  required_approving_review_count: 1
  dismiss_stale_reviews: true
restrictions: null
```

## Workflow Triggers

### CI Pipeline Triggers
- **Push**: `main`, `develop` branches
- **Pull Request**: `main`, `develop` branches
- **Schedule**: Benchmarks run on main branch pushes

### ESP32 Pipeline Triggers
- **Push**: Changes to ESP32-related files
- **Pull Request**: Changes to ESP32-related files
- **Paths**: `_examples/esp32/**`, `src/esp32_miner.rs`

### Documentation Triggers
- **Push**: `main` branch (source code or markdown changes)
- **Pull Request**: `main` branch (markdown changes only)

## Artifacts and Outputs

### Build Artifacts
- **Linux x64**: `tribechain-linux-x64.tar.gz`
- **Windows x64**: `tribechain-windows-x64.zip`
- **macOS x64**: `tribechain-macos-x64.tar.gz`
- **macOS ARM64**: `tribechain-macos-arm64.tar.gz`
- **ESP32 Firmware**: `esp32-firmware-{target}.bin`

### Docker Images
- **Registry**: `bittribe/tribechain`
- **Tags**: `latest`, `main`, `develop`, `{branch}-{sha}`
- **Platforms**: `linux/amd64`, `linux/arm64`

### Documentation
- **API Docs**: Rust documentation with private items
- **User Guide**: mdBook-generated comprehensive guide
- **ESP32 Docs**: Microcontroller-specific documentation

## Performance Monitoring

### Benchmarks
- **Frequency**: On every push to `main`
- **Storage**: GitHub Pages with historical tracking
- **Alerts**: 200% performance regression threshold
- **Tool**: Criterion.rs benchmarks

### Coverage Tracking
- **Provider**: Codecov
- **Format**: LCOV
- **Scope**: Entire workspace
- **Reporting**: PR comments and status checks

## Maintenance

### Regular Tasks
- **Dependency updates**: Automated via Dependabot
- **Security audits**: Weekly via `cargo audit`
- **Performance monitoring**: Continuous via benchmarks
- **Documentation updates**: Automatic on source changes

### Manual Interventions
- **Release creation**: Tag with `v*` pattern
- **Security fixes**: Manual dependency updates if needed
- **Workflow updates**: Periodic review and updates

## Troubleshooting

### Common Issues

**Build Failures:**
- Check system dependencies (libclang-dev, etc.)
- Verify Rust version compatibility
- Review Cargo.lock conflicts

**Docker Build Issues:**
- Ensure Dockerfile is up to date
- Check multi-platform build compatibility
- Verify base image availability

**ESP32 Build Problems:**
- Confirm PlatformIO installation
- Check target board compatibility
- Verify library dependencies

**Documentation Failures:**
- Check mdBook installation
- Verify markdown syntax
- Review link validity

### Getting Help

1. Check workflow logs in the Actions tab
2. Review artifact outputs for detailed information
3. Consult individual workflow files for configuration
4. Open an issue for persistent problems

## Contributing

When modifying workflows:

1. Test changes in a fork first
2. Update this documentation
3. Consider backward compatibility
4. Add appropriate comments in workflow files
5. Test with different trigger conditions

---

For more information about GitHub Actions, see the [official documentation](https://docs.github.com/en/actions). 