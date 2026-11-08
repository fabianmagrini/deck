# workbench-git

Git integration for the Deck AI Agent Workbench. **Stubbed — planned for Milestone 3/4.**

## Planned scope

This crate will expose workspace-level Git status and unified diff information so the inspector panel can display what files an agent has changed and allow the developer to review them before committing.

Planned modules:

| Module | Description |
|--------|-------------|
| `status` | Working-tree file status (modified, added, deleted, untracked) |
| `diff` | Unified text diffs for changed files |

The implementation will use `git2` (libgit2 bindings) or shell out to the `git` CLI behind a trait so the backend can be swapped.

## Security note

File inspection is restricted to the opened workspace directory. Symlinks and canonical paths must be resolved safely to prevent directory traversal.

## Current status

Both modules compile as empty stubs. Git functionality is not yet wired into the UI.
