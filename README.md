<div align="center">
  <br />
  <h1>trek</h1>
  <p><strong>Opinionated scaffolding, validation, and packaging CLI for FiveM resource development.</strong></p>
</div>

---

## Features

- **Interactive & Non-interactive Scaffolding**: Generate standard FiveM resource structures with multi-framework boilerplate (ESX, QBCore, Qbox) in seconds.
- **Pattern-based Resource Packaging**: Package resources into production-ready `.zip` release archives using `.pack` allowlist patterns.
- **Contract-Driven NUI Codegen**: Generate typed React NUI hooks and Lua helpers from `nui-schema.yaml` with enum and RPC support.
- **Ultra-lean Binary**: Optimized Rust build compiled for minimal binary footprint.

---

## Installation & Build

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (edition 2024 / stable)

### Build from Source

This repository is a Cargo workspace monorepo. The CLI crate lives at `crates/trek`.

```bash
# Build optimized release binary
cargo build --release

# The compiled binary is located at target/release/trek
# Optionally install globally to your cargo bin path:
cargo install --path crates/trek
```

---

## Usage & Commands

```bash
trek [COMMAND]
```

### 1. `generate`

Scaffolds a new FiveM resource directory with configuration files, client/server scripts, and shared utilities.

```bash
trek generate [OPTIONS]
```

#### Options & Flags

| Flag / Option | Short | Type | Description |
| :--- | :--- | :--- | :--- |
| `--name <NAME>` | `-n` | `String` | Resource name. If omitted, triggers interactive prompt. |
| `--description <DESC>` | `-d` | `String` | Resource description in `fxmanifest.lua`. If omitted, defaults to `"A FiveM resource for <NAME>"`. |
| `--frameworks <LIST>...` | `-f` | `List` | Framework integrations to include (`ESX`, `QBCore`, `Qbox`, `None`). Multiple values supported. |
| `--help` | `-h` | | Print help information. |

#### Examples

**Interactive Mode:**
```bash
trek generate
```
*Prompts interactively for resource name, description, and framework selections.*

**Non-Interactive / CLI Flags:**
```bash
# Generate a resource with ESX and QBCore support
trek generate -n my-resource -d "Custom vehicle shop" -f ESX QBCore

# Generate a standalone resource
trek generate -n simple-teleport -f None
```

#### Generated Directory Layout

```text
my-resource/
├── .pack           # Allowlist patterns for release packaging
├── fxmanifest.lua       # FiveM resource manifest
├── config/
�│   ├── client.lua       # Client-side configuration
�│   ├── server.lua       # Server-side configuration
�│   └── share.lua        # Shared configuration
└── src/
�    ├── client/
�   �│   ├── client.lua   # Client-side logic
�    ├── server/
�   �│   ├── server.lua   # Server-side logic
�    └── shared/
�        └── utils.lua    # Shared utility functions
```

---

### 2. `pack`

Packages the current FiveM resource into a compressed release `.zip` file using the include patterns defined in `.pack`.

```bash
trek pack [OPTIONS]
```

#### Options & Flags

| Flag / Option | Short | Type | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-o <OUT_DIR>` | `-o` | `Path` | `.` | Output directory where the zip archive is saved. |
| `--dry-run` | | `bool` | `false` | Simulate packing without creating or writing the `.zip` archive. |
| `--report` | | `bool` | `false` | Output a detailed summary report in Markdown format. |
| `--sha256` | | `bool` | `false` | Print the SHA-256 checksum of the archive and include it in the report. |
| `--help` | `-h` | | | Print help information. |

#### Examples

```bash
# Run inside your resource directory:
cd my-resource

# Pack resource into ./my-resource.zip
trek pack

# Dry run simulation with timing
trek pack --dry-run

# Pack and output a Markdown summary report
trek pack --report

# Pack to a specific directory with markdown report
trek pack -o ./dist --report
```

#### `.pack` File Format

The `.pack` file in the root of your resource specifies which files to include in the release archive using glob patterns. Lines starting with `#` and empty lines are ignored:

```text
# trek include patterns
fxmanifest.lua
config/**/*.lua
src/**/*.lua
```

---

### 3. `version`

Shows the current resource version or automates SemVer version bumping in `fxmanifest.lua` while preserving the exact syntax style (e.g. `version '1.0.0'`, `version("1.0.0")`, or `version = "1.0.0"`).

```bash
trek version [OPTIONS]
```

#### Options & Flags

| Flag / Option | Short | Type | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `--patch` | | `bool` | | Bump patch version (e.g., `1.2.3` -> `1.2.4`). |
| `--minor` | | `bool` | | Bump minor version and reset patch (e.g., `1.2.3` -> `1.3.0`). |
| `--major` | | `bool` | | Bump major version and reset minor/patch (e.g., `1.2.3` -> `2.0.0`). |
| `--ci` | | `bool` | | Print only the raw version, no styling (machine-readable). |
| `-m, --manifest <PATH>` | `-m` | `Path` | `fxmanifest.lua` | Path to the target `fxmanifest.lua`. |
| `--help` | `-h` | | | Print help information. |

> **Note:** Without a bump flag, the current version is printed without modifying the manifest. At most one increment flag (`--major`, `--minor`, or `--patch`) may be given per execution.

#### Examples

```bash
# Show current version (read-only)
trek version

# Print only the raw version (for scripts / CI)
trek version --ci
# -> 1.2.3

# Bump patch version (1.0.0 -> 1.0.1)
trek version --patch

# Bump minor version (1.0.1 -> 1.1.0)
trek version --minor

# Bump major version (1.1.0 -> 2.0.0)
trek version --major

# Operate on a specific resource directory
trek version --patch -m ./my-resource/fxmanifest.lua
```

---

### 4. `validate`

Lints `fxmanifest.lua` for common problems using the built-in manifest parser.

Checks:
- Required declarations present (`fx_version`, `game`)
- `lua54 'yes'` recommended
- Script/file entries exist on disk (relative to the manifest)
- Duplicate entries within a script list
- Framework imports (`@es_extended`, `@qb-core`, ...) declared under `dependency`/`dependencies`

```bash
trek validate [OPTIONS]
```

| Flag / Option | Short | Type | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `-m, --manifest <PATH>` | `-m` | `Path` | `fxmanifest.lua` | Path to the target `fxmanifest.lua`. |

Exits non-zero when any error-level finding is reported (CI-friendly); warnings alone do not fail.

#### Examples

```bash
# Validate the current resource
trek validate

# Validate a specific manifest
trek validate -m ./my-resource/fxmanifest.lua
```

---

### 5. `codegen`

Generates typed React NUI hooks and Lua NUI helpers from a YAML contract (`nui-schema.yaml`). Supports enums, one-way events, and two-way RPC endpoints (`query` / `mutation`).

```bash
# Create starter nui-schema.yaml and trek-nui.schema.json
trek codegen --init-schema

# Generate the default React and Lua bindings
trek codegen

# Use custom paths
trek codegen -s ./nui-schema.yaml -t ./react/src/generated/nui.ts -l ./src/shared/nui_events.lua
```

#### Example `nui-schema.yaml`

```yaml
version: "1.0"
resource: "my_resource"

enums:
  - name: PlayerStatus
    description: "Current player status"
    values:
      - idle
      - in_combat
      - dead

events:
  - name: statusUpdated
    description: "Emitted when player status changes"
    payload:
      status: PlayerStatus
      health: number

endpoints:
  - name: getPlayerStatus
    type: query
    response:
      status: PlayerStatus
```

> For the full schema specification, type system details, and end-to-end examples, see the [nui-schema.yaml Guide](docs/nui-schema.md).

---

### 6. `release`

Composite command that runs the full release pipeline: **validate → bump → pack**.

Aborts before packing if validation reports any error. Version bumping is optional � without a bump flag the current version is kept. The Markdown report is always included, optionally with the archive's SHA-256 checksum.

```bash
trek release [OPTIONS]
```

| Flag / Option | Short | Type | Default | Description |
| :--- | :--- | :--- | :--- | :--- |
| `--patch` | | `bool` | | Bump patch version (e.g., `1.2.3` -> `1.2.4`) before packing. |
| `--minor` | | `bool` | | Bump minor version and reset patch (e.g., `1.2.3` -> `1.3.0`). |
| `--major` | | `bool` | | Bump major version and reset minor/patch (e.g., `1.2.3` -> `2.0.0`). |
| `-o <OUT_DIR>` | `-o` | `Path` | `.` | Output directory where the zip archive is saved. |
| `-m, --manifest <PATH>` | `-m` | `Path` | `fxmanifest.lua` | Path to the target `fxmanifest.lua`. |
| `--sha256` | | `bool` | `false` | Print the SHA-256 checksum of the archive and include it in the report. |

#### Examples

```bash
# Validate + pack with report (no version bump)
trek release

# Release with patch bump and SHA-256 checksum
trek release --patch --sha256

# Minor release into a dist folder
trek release --minor -o ../dist
```

---

## License

MIT / Apache-2.0