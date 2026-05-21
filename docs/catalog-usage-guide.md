# Catalog Usage Guide for AI Agents

This guide explains how to query `repo_index.db` so that generated code, schemas, and playbooks stay aligned with the existing repository catalog.

## 1. Catalog overview

- `repo_index.db` is a SQLite database at the repository root.
- It indexes:
  - Schemas (JSON Schema, ALN shards, YAML schemas).
  - Playbooks and tools (shell scripts, Rust binaries, examples).
  - Crates and binaries (Rust crates, features, examples).
  - Documents (design notes, policies, calibration protocols).
- It also stores relationships:
  - Which playbooks depend on which schemas.
  - Which crates own which binaries or tools.
  - Which files implement or validate a given schema.

This mirrors the way your eco‑placement calibration pipeline and biomem tooling tie JSON/YAML schemas to Rust modules and ALN shards.[file:12][file:13][file:19]

## 2. Connecting to the catalog

- Use SQLite:
  - CLI: `sqlite3 repo_index.db`.
  - Rust: `rusqlite` or `sqlx`.
  - AI agents: issue text‑only SQL queries that are safe and read‑only.

- Recommended pragma for agents:
  - `PRAGMA query_only = ON;`
  - `PRAGMA foreign_keys = ON;`

These match the integrity guarantees used in your calibration and biomem databases.[file:13][file:19]

## 3. Core tables

Exact table names may vary slightly; agents should always introspect with `.schema` first.

Typical tables:

- `schemas`:
  - `id INTEGER PRIMARY KEY`
  - `name TEXT` (e.g., `hardware-catalog.v1`)
  - `kind TEXT` (`json-schema`, `aln`, `yaml-schema`)
  - `path TEXT` (relative file path)
  - `version TEXT`
  - `tags TEXT` (comma-separated or JSON)

- `playbooks`:
  - `id INTEGER PRIMARY KEY`
  - `name TEXT` (e.g., `eco-placement-dryrun`)
  - `path TEXT` (e.g., `tools/playbook_simulate.sh`)
  - `kind TEXT` (`shell`, `rust-bin`)
  - `description TEXT`

- `crates`:
  - `id INTEGER PRIMARY KEY`
  - `name TEXT` (e.g., `aln-core`)
  - `path TEXT` (crate directory)
  - `kind TEXT` (`lib`, `bin`, `proc-macro`)
  - `manifest_path TEXT` (`crates/aln-core/Cargo.toml`)

- `documents`:
  - `id INTEGER PRIMARY KEY`
  - `name TEXT` (e.g., `eco-calibration-security.v1`)
  - `path TEXT`
  - `category TEXT` (`policy`, `design`, `spec`)

These map directly to the artifacts referenced throughout your research notes (hardware catalog, grid carbon catalog, playbooks, policies).[file:12][file:13]

## 4. Relationship tables

- `schema_references`:
  - links schemas to other schemas or documents.
  - columns: `from_schema_id`, `to_schema_id`, `relation` (`extends`, `validates`, `uses`).

- `playbook_dependencies`:
  - links playbooks to schemas, crates, or tools.
  - columns: `playbook_id`, `target_type`, `target_id`, `relation`.

- `crate_artifacts`:
  - links crates to binaries/examples/schemas.
  - columns: `crate_id`, `artifact_type`, `artifact_path`.

This matches how your eco‑placement CLI references grid‑carbon, hardware, and workload schemas, and how biomem examples reference ALN shards.[file:12][file:19]

## 5. Discovering schemas

For an agent generating code against existing schemas:

- List all schemas:

  ```sql
  SELECT name, kind, path, version
  FROM schemas
  ORDER BY name;
  ```

- Find a schema by name prefix:

  ```sql
  SELECT name, path
  FROM schemas
  WHERE name LIKE 'hardware-catalog.v1%';
  ```

- Find all schemas that validate a given config file:

  ```sql
  SELECT s.name, s.path
  FROM schemas s
  JOIN schema_references r ON r.to_schema_id = s.id
  WHERE r.relation = 'validates'
    AND r.from_schema_id = (
      SELECT id FROM schemas WHERE name = 'hardware-catalog.v1'
    );
  ```

Agents should always use these paths rather than inventing new filenames.[file:12][file:13]

## 6. Discovering playbooks and tools

To understand the operational surface:

- List playbooks:

  ```sql
  SELECT name, path, kind, description
  FROM playbooks
  ORDER BY name;
  ```

- Find all playbooks that touch a given schema:

  ```sql
  SELECT p.name, p.path, d.relation
  FROM playbooks p
  JOIN playbook_dependencies d ON d.playbook_id = p.id
  JOIN schemas s ON s.id = d.target_id
  WHERE d.target_type = 'schema'
    AND s.name = 'ble-session.v1';
  ```

- Discover CI‑relevant simulations:

  ```sql
  SELECT p.name, p.path
  FROM playbooks p
  WHERE p.kind = 'shell'
    AND p.name LIKE '%simulate%';
  ```

This is analogous to how you manage eco‑placement dry‑runs and biomem PoF examples.[file:12][file:19]

## 7. Mapping crates to artifacts

Agents must reuse existing crates instead of generating duplicate functionality.

- Find crate path and manifest:

  ```sql
  SELECT name, path, manifest_path
  FROM crates
  WHERE name = 'aln-core';
  ```

- Find examples or binaries attached to a crate:

  ```sql
  SELECT c.name, a.artifact_type, a.artifact_path
  FROM crates c
  JOIN crate_artifacts a ON a.crate_id = c.id
  WHERE c.name = 'ble-governance';
  ```

- Cross‑check that a crate already owns a tool:

  ```sql
  SELECT c.name
  FROM crates c
  JOIN crate_artifacts a ON a.crate_id = c.id
  WHERE a.artifact_path = 'tools/run_all_checks.sh';
  ```

This prevents agents from creating overlapping binaries when a crate already implements a feature.[file:19]

## 8. Using the catalog when generating code

When an AI agent proposes new code:

- Always:
  - Query `schemas` for existing definitions before inventing new ones.
  - Query `playbooks` to see if a similar workflow exists.
  - Query `crates` and `crate_artifacts` to find the right home for new code.

- For new schemas:
  - Reuse version families (`*.v1`, `*.v2`) and naming conventions present in `schemas`.
  - Add catalog entries via seeding tools instead of editing `repo_index.db` directly.

- For new playbooks/tools:
  - Attach them to existing crates or schema families via `playbook_dependencies`.
  - Ensure they validate against the right `*.schema.json` entries.

This is the same governance pattern used for eco‑placement calibration catalogs and biomem shards.[file:12][file:13][file:19]

## 9. Safe query patterns for agents

Agents should:

- Use only `SELECT` and `PRAGMA query_only = ON`.
- Avoid:
  - `INSERT`, `UPDATE`, `DELETE`.
  - DDL statements.

If write access is needed, it must happen through dedicated Rust or shell seeding tools that encode governance constraints, as in your eco‑calibration workflows.[file:12]

## 10. Example: locating a schema for validation

To validate `.ble-environment.aln`:

1. Find the session schema:

   ```sql
   SELECT path
   FROM schemas
   WHERE name = 'ble-session.v1';
   ```

2. Discover which tools reference it:

   ```sql
   SELECT p.name, p.path
   FROM playbooks p
   JOIN playbook_dependencies d ON d.playbook_id = p.id
   JOIN schemas s ON s.id = d.target_id
   WHERE s.name = 'ble-session.v1';
   ```

3. Use those playbooks (e.g., `generate_environment_sample.sh`) as the canonical path for validation.

This keeps code and tooling aligned with the indexed catalog rather than ad‑hoc files.[file:13][file:19]
