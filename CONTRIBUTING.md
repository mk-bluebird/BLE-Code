# Contributing to BLE‑Code

Thank you for considering contributing! This project is governed by strict neurorights and RoH policies. Please read the governance documents (`repo-governance.aln`, `AiContributionPolicy.aln`) before contributing.

## Development process

1. **Fork & branch.** Create a feature branch.
2. **Ensure CI is green.** Our CI runs `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`, and governance lints.
3. **Do not weaken safety.** Pull requests must not:
   - Remove required CI jobs.
   - Bypass or disable the BLE guard for real actuation.
   - Widen RoH ceilings beyond 0.3.
   - Introduce unsafe code in core crates.
4. **AI contributions:** Mark AI‑authored crates with `ai-authored` in their `Cargo.toml` metadata (under `[package.metadata.aln] ai-authored = true`) and ensure they pass the AI Clippy profile.
5. **Commit messages:** Follow conventional commits (`feat:`, `fix:`, `chore:`, `docs:`).

## Sign‑off

By submitting a PR, you affirm that your contribution complies with the project’s neurorights and governance rules. For AI agents, this is your sovereign coding contract.
