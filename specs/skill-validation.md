# Skill Validation Spec

## One-Sentence Description

Skill validation verifies that skill directories conform to the SKILL.md specification.

## Overview

The `validate` command checks a skill's SKILL.md file against format and content requirements. Validation runs a sequence of checks, stopping early if a blocking check fails. Results include pass/fail status, detected metadata, warnings for missing optional fields, and detected directory structure.

## Input Resolution

The `validate` command accepts either a file path or an installed skill name:

| Input Type | Example | Resolution |
|------------|---------|------------|
| Directory path | `/path/to/skill` | Uses path directly |
| Relative path | `./my-skill` | Resolves to absolute path |
| Installed skill name | `my-skill` | Looks up skill in installed skills directory via Scanner |

When given a skill name (not a path), the command uses `Scanner` to find the installed skill's location. This allows validating skills after installation without knowing their exact path.

## Validation Rules

Validation runs these checks in order:

| Check | Blocking | Description |
|-------|----------|-------------|
| SKILL.md exists | Yes | File must exist in skill directory |
| YAML frontmatter is valid | Yes | Must have `---` delimiters at file start |
| Required fields present | Yes | `name` and `description` must be present |
| Name format is valid | No | Must match naming pattern |
| Description length is valid | No | Must be 1-1024 characters |

Blocking checks stop further validation if they fail.

## SKILL.md Requirements

### Required Fields

- **name**: Primary identifier for the skill
- **description**: Human-readable description (1-1024 characters)

### Optional Fields

Missing optional fields generate warnings (not errors):

- **version**: Version string
- **author**: Author name
- **license**: License identifier

### Frontmatter Format

```yaml
---
name: my-skill
description: A useful skill
version: 1.0.0
author: Author Name
license: MIT
---
```

Frontmatter must:
- Start at the beginning of the file (only whitespace allowed before first `---`)
- Use two `---` delimiters
- Contain valid YAML between delimiters

## Name Validation

Skill names must match pattern: `^[a-z0-9][a-z0-9_-]{0,63}$`

| Rule | Valid | Invalid |
|------|-------|---------|
| Start with lowercase letter or digit | `my-skill`, `0skill` | `-skill`, `_skill` |
| Contain only `a-z`, `0-9`, `-`, `_` | `my_skill-1` | `My.Skill`, `my skill` |
| Length 1-64 characters | `a`, 64-char string | empty, 65+ chars |
| No path separators | `skill` | `my/skill`, `my\skill` |
| Not path traversal | `skill` | `.`, `..` |

## Validation Output

### Human-Readable Format

```
Validating skill at: /path/to/skill

✓ SKILL.md exists
✓ YAML frontmatter is valid
✓ Required fields present
✓ Name format is valid
✓ Description length is valid (1-1024)

Metadata:
  name: my-skill
  version: 1.0.0
  author: Author Name
  license: MIT

Detected directories:
  scripts/: ✓
  references/: ✗

PASSED
```

### JSON Format

```json
{
  "passed": true,
  "skill_path": "/path/to/skill",
  "checks": [
    { "name": "SKILL.md exists", "passed": true },
    { "name": "YAML frontmatter is valid", "passed": true },
    { "name": "Required fields present", "passed": true },
    { "name": "Name format is valid", "passed": true },
    { "name": "Description length is valid (1-1024)", "passed": true }
  ],
  "warnings": ["Optional field 'version' is missing"],
  "metadata": {
    "name": "my-skill",
    "description": "A useful skill"
  },
  "detected_directories": {
    "has_scripts": true,
    "has_references": false
  }
}
```

## Error Messages

| Error | Cause |
|-------|-------|
| `SKILL.md file not found` | SKILL.md doesn't exist |
| `missing frontmatter delimiters (no '---' markers found)` | No `---` in file |
| `malformed frontmatter (only one '---' marker found, expected two)` | Single `---` marker |
| `frontmatter must be at the start of the file` | Content before first `---` |
| `missing required field 'name'` | No name field |
| `missing required field 'description'` | No description field |
| `invalid skill name '...'` | Name fails pattern check |
| `skill name cannot be empty` | Empty name |
| `skill name cannot contain path separators` | `/` or `\` in name |
| `Path traversal detected` | Name is `.` or `..` |
| `Description is empty` | Empty description |
| `Description is too long: N characters (max 1024)` | Description > 1024 chars |

## Dependencies

- `src/core/parser.rs`: `extract_frontmatter`, `parse_skill_md`, `validate_skill_name`
- `src/core/scanner.rs`: `Scanner` for resolving installed skill names
- `src/core/config.rs`: `Config` for agent configuration
- `src/core/errors.rs`: `SikilError` types
- `src/cli/output.rs`: `Output` for formatted output

## Used By

- CLI `validate` command (`sikil validate <path-or-name>`)
- Potentially by `install` command for pre-installation validation
