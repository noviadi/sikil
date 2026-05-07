# Provenance Spec

## One-Sentence Description

Provenance records the source origin of every managed skill through per-skill sidecar files.

## Overview

Provenance is the per-skill record of where a managed skill came from: the source URL, the resolved commit, the content hash, and the install timestamp. For project-scoped skills the equivalent data lives in `.sikil/lock.toml` (see [project-manifest.md](project-manifest.md)); for global skills there is no manifest, so provenance is recorded as a sidecar TOML file inside each managed skill directory at `~/.sikil/repo/<name>/.sikil-source.toml`. Provenance enables audit ("where did this come from?"), update ("what should I re-fetch?"), and integrity verification ("does this match what I installed?").

## Sidecar Location

```
~/.sikil/repo/<skill-name>/
├── .sikil-source.toml    # provenance sidecar
├── SKILL.md              # the skill itself
├── scripts/              # optional
└── references/           # optional
```

The sidecar lives **inside** the skill directory so it travels with the bytes when the directory is moved or copied. Skill consumers (the agents) ignore files starting with `.` so the sidecar is invisible to them.

## Sidecar Schema (`.sikil-source.toml`)

```toml
version = 1
source = "<source-as-installed>"
resolved_url = "<expanded-url-or-absolute-path>"
commit = "<full-sha>"
content_hash = "sha256:<hex>"
installed_at = "<rfc3339-timestamp>"
installer = "sikil/<version>"
```

### Field Definitions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | integer | Yes | Sidecar format version; currently `1` |
| `source` | string | Yes | Verbatim source argument passed to `sikil install` (short form, URL, or local path) |
| `resolved_url` | string | Yes | Fully expanded clone URL or absolute local path |
| `commit` | string | Yes for git sources | Full git commit SHA; empty string for local-path sources |
| `content_hash` | string | Yes | SHA-256 of the skill tree excluding `.git/` and `.sikil-source.toml` itself; prefixed `sha256:` |
| `installed_at` | string | Yes | RFC 3339 UTC timestamp of installation |
| `installer` | string | Yes | Version string in the form `sikil/<semver>`; recorded for diagnostic purposes |

The sidecar uses `#[serde(deny_unknown_fields)]` to reject malformed entries.

## When Sidecars Are Written

A `.sikil-source.toml` file is written:

1. After every successful `sikil install <source> --global` (or `sikil install <source>` outside any project)
2. After `sikil adopt <name>` for a skill adopted into the global repo
3. Never automatically rewritten by read-only commands (`list`, `show`, `validate`, scanner)

Sidecars for legacy global skills (installed before this feature) are missing. Sikil treats their absence as `unknown_origin`; see "Unknown Origin Handling" below.

## When Sidecars Are Read

Sidecars are read by:

| Consumer | Use |
|----------|-----|
| `sikil show <name>` | Display source, commit, installed_at in the human-readable detail view |
| `sikil list --json` | Include source / commit fields per skill installation |
| `sikil update <name>` (global mode) | Locate the source to re-fetch from |
| Future `sikil audit` | Report unknown-origin and content-hash mismatches |

Sidecars are **not** read by `sikil install` for materialization — the bytes are the source of truth for what's installed; the sidecar is metadata about how those bytes got there.

## Atomicity

Sidecar writes use atomic temp-file + rename via [atomic-operations.md](atomic-operations.md). A crash mid-write leaves either the previous sidecar (if any) or no sidecar; never a partial file.

The sidecar write is a **separate step** from the skill content write. If the content copy succeeds but the sidecar write fails, the install reports a non-fatal warning (`"skill installed but provenance sidecar could not be written: <reason>"`) and exits successfully. The skill is usable; only the audit trail is missing. The user can re-run `sikil install --reinstall <name>` to retry the sidecar (deferred to v0.3).

## Unknown Origin Handling

Skills in `~/.sikil/repo/<name>/` without a `.sikil-source.toml` are classified as **unknown origin**. They appear in `sikil list` and `sikil show` with the marker `(unknown origin)` in human-readable mode and `"source": null` in JSON mode.

`sikil update <name>` on an unknown-origin skill returns `ValidationError` with the message:

```
cannot update '<name>': source is unknown.
Reinstall with `sikil install <source> --global` to record provenance.
```

This is intentional: we never guess where a skill came from. The user must either reinstall (which records provenance) or accept that the skill is locally-owned and not updatable.

## Content Hash Computation

The `content_hash` is computed deterministically:

1. Walk the skill directory with `WalkDir::follow_links(false)`
2. Exclude `.git/` (any depth) and the sidecar file itself (`.sikil-source.toml` at depth 0)
3. Sort files by their path relative to the skill root (stable cross-platform ordering)
4. For each file: hash its relative path bytes (UTF-8), a single `0x00` separator, then its content bytes
5. Final hash is the SHA-256 hex digest of the concatenated stream, prefixed with `sha256:`

The same algorithm is used for project lockfile `content_hash` per [project-manifest.md](project-manifest.md), so a skill installed both globally and in a project will have identical hashes if the bytes match.

## Symmetry With Project Lockfile

| Concern | Project skills | Global skills |
|---------|---------------|---------------|
| Where intent lives | `.sikil/manifest.toml` | (none — accreted state) |
| Where resolved state lives | `.sikil/lock.toml` (one file, all skills) | `~/.sikil/repo/<name>/.sikil-source.toml` (per skill) |
| Schema fields | identical (`source`, `commit`, `content_hash`, etc.) | identical |
| Rewriter | `sikil install`, `sikil update` | `sikil install`, `sikil adopt` |
| Reader | `sikil install` (reconciliation), `sikil update` | `sikil show`, `sikil update`, future `audit` |

The asymmetry (central lockfile for projects vs per-skill sidecar for global) reflects the genuine semantic difference: a project declares its skill set and wants one-file aggregate review; the global repo accretes skills over time and benefits from provenance-travels-with-bytes locality.

## Acceptance Criteria

- After `sikil install <source> --global`, `~/.sikil/repo/<name>/.sikil-source.toml` exists with all required fields populated
- `source` field in the sidecar matches verbatim the argument passed to `sikil install`
- `resolved_url` field is the expanded clone URL or an absolute local path
- `commit` field is the full git commit SHA for git sources, empty string for local-path sources
- `content_hash` field is `sha256:<hex>` and verifiably matches the SHA-256 of the skill bytes (excluding `.git/` and the sidecar)
- `installed_at` field is a valid RFC 3339 UTC timestamp
- `installer` field is `sikil/<version>` matching the running binary's version
- Sidecar is written atomically via temp file + rename
- Sidecar write failure after successful content copy is non-fatal: install reports a warning and exits 0
- Sidecar write failure produces a warning containing the substring "provenance sidecar"
- `sikil show <name>` displays source, commit, and installed_at when sidecar present
- `sikil show <name>` displays `(unknown origin)` when sidecar absent
- `sikil list --json` per-installation entry contains `"source": "<source>"` when sidecar present
- `sikil list --json` per-installation entry contains `"source": null` when sidecar absent
- `sikil update <name>` on unknown-origin skill returns `ValidationError` with the substring "source is unknown"
- `sikil update <name>` on a known-origin skill re-fetches from `resolved_url` at the original `source`'s ref-style (recorded by re-reading sidecar's `source` field; semver-style refs are not implemented in v0.2)
- Sidecar with `version` other than `1` returns `ValidationError` with substring "unsupported sidecar version"
- Sidecar with unknown fields returns `ValidationError` (deny_unknown_fields)
- Sidecar excluded from `content_hash` computation (the hash is stable across reads even if the sidecar changes timestamp)
- `.git/` directory excluded from `content_hash` computation at any depth
- Content hash computed for the same skill bytes is identical between global sidecar and project lockfile

## Error Conditions

| Condition | Error Type | Severity |
|-----------|------------|----------|
| Sidecar file missing | (none) | Treated as unknown origin |
| Sidecar TOML invalid | `ValidationError` | Surface in `sikil show`; do not block scan |
| Sidecar `version` unsupported | `ValidationError` | Surface in `sikil show` |
| Sidecar unknown fields | `ValidationError` | Surface in `sikil show` |
| Sidecar write fails after content copy | (warning only) | Install exits 0 with warning |
| `sikil update` on unknown-origin skill | `ValidationError` | Exit code 2 |

## Dependencies

| Component | Purpose |
|-----------|---------|
| `utils::atomic::atomic_write_file` | Atomic sidecar write |
| `sha2` | Content hash computation |
| `walkdir` | Deterministic file ordering for hash |
| `toml` | Parse and serialize sidecar |
| `serde` with `deny_unknown_fields` | Schema enforcement |
| `chrono` or equivalent | RFC 3339 timestamp formatting |

## Used By

| Consumer | Usage |
|----------|-------|
| `commands::install` | Writes sidecar after successful global install |
| `commands::adopt` | Writes sidecar when adopting into global repo (source = original installation path) |
| `commands::show` | Reads sidecar to display provenance |
| `commands::list` | Reads sidecar for `--json` output |
| `commands::update` | Reads sidecar to re-fetch from recorded source |
| Future `commands::audit` | Reports unknown-origin and content-hash mismatches |
