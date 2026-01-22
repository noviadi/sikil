# Agent Skills Manager - Comprehensive Brainstorm

**Note:** This folder is research-only and non-authoritative. For execution and implementation, refer to `PRD.md`, `TRD.md`, `implementation_roadmap.md`, `use_cases.md`, and `traceability_matrix.md` at the repo root.

## Overview

This brainstorm explores the problem of **managing Agent Skills across multiple coding agents** and proposes a comprehensive CLI/TUI solution to bring visibility, control, and automation to the fragmented skills ecosystem.

## The Problem

You're using 5+ coding agents (Claude Code, Windsurf, OpenCode, Kilo Code, Amp Code), and each one stores skills in different locations:

```
~/.claude/skills/
~/.codeium/windsurf/skills/
~/.config/opencode/skill/
~/.kilocode/skills/
~/.config/agents/skills/
+ 10+ project-level directories

Result: 47 skills, 15+ directories, 7+ duplicates, 3+ version conflicts
```

**Key Pain Points:**
- 🔴 No unified visibility of what skills exist where
- 🔴 Version mismatches across agents (same skill, different versions)
- 🔴 Duplicate installations waste disk space and maintenance effort
- 🔴 Manual installation across agents is error-prone
- 🔴 Cleanup and updates require touching multiple directories
- 🔴 Onboarding new developers takes hours of manual setup
- 🔴 Impossible to track which skills are actually used

## The Solution: Unified Skills Manager

A **CLI-first tool** with optional TUI that:

✅ **Discovers** all skills across all agents automatically  
✅ **Catalogs** them in a queryable database with full metadata  
✅ **Syncs** skills across agents with one command  
✅ **Validates** SKILL.md format and detects conflicts  
✅ **Manages** installations, removals, and cleanup safely  
✅ **Reports** on usage, versions, and issues  

## Documents Included

### 1. **agent_skills_research.md**
Comprehensive research on Agent Skills, the specification, and how each of the 5 major agents implements it.

### 2. **agent-skills-cli.md** ⭐ START HERE
The complete brainstorm document with detailed feature set, command examples, TUI mockups, and implementation roadmap.

### 3. **agent_skills_comparison_matrix.md**
Side-by-side comparison of all 5 agents with detailed path structures and decision matrices.

### 4. **agent_skills_technical_spec.md**
Technical architecture, data models, and implementation details for developers.

### 5. **QUICK_START.md**
TL;DR guide for different audiences.

## Key Insights

### The Fragmentation Problem is Real

| Aspect | Impact |
|--------|--------|
| **15+ directories** | Developer confusion, scattered knowledge |
| **7+ duplicate skills** | Wasted disk space, maintenance burden |
| **Version mismatches** | Agent inconsistency, broken workflows |
| **Manual sync** | 2-4 hour onboarding, high error rate |
| **No visibility** | Can't audit or analyze skill usage |

## Getting Started

**For understanding**: Read `agent-skills-cli.md` (30 min)
**For development**: Read `agent_skills_technical_spec.md`
**For details**: Review `agent_skills_comparison_matrix.md`

All documents are cross-referenced and ready to share.

---
**Created**: January 16, 2026  
**Status**: Ready for implementation