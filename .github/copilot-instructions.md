# rust-task-manager Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-10-15

## Active Technologies
- Rust 1.70+ (2021 edition) for backend, TypeScript 5.x with strict mode for frontend (001-build-a-modern)

## Project Structure
```
src-tauri/          # Rust backend
  src/              # Rust source files
  Cargo.toml        # Rust dependencies
src/                # TypeScript frontend
  types/            # TypeScript type definitions
  services/         # API service layer
  ui/               # UI components
specs/              # All specification documents
  001-build-a-modern/
    spec.md         # Feature specification
    plan.md         # Implementation plan
    tasks.md        # Task breakdown
    data-model.md   # Data structures
    contracts/      # API contracts
```

## Commands
cargo test; cargo clippy

## Code Style
Rust 1.70+ (2021 edition) for backend, TypeScript 5.x with strict mode for frontend: Follow standard conventions

## Documentation Policy
- **ONLY update existing files in specs/ folder**: spec.md, plan.md, tasks.md, data-model.md, contracts/
- **DO NOT create phase summary documents** (e.g., PHASE1_COMPLETE.md, PHASE1_SUMMARY.md)
- **Update tasks.md directly**: Mark tasks complete with [x] and add validation notes inline
- **Update CHANGELOG.md**: Document completed phases and changes
- **Avoid duplicate documentation**: All phase progress should be tracked in tasks.md only

## Workflow for Phase Completion
1. Complete all tasks in the phase
2. Run validation commands (cargo check, clippy, fmt, npm lint)
3. Update tasks.md: Mark tasks [x] and add brief validation results
4. Update CHANGELOG.md: Add phase completion entry
5. DO NOT create separate phase completion documents

## Recent Changes
- 001-build-a-modern: Added Rust 1.70+ (2021 edition) for backend, TypeScript 5.x with strict mode for frontend
- Phase 1 (Setup): Completed all 8 tasks, all validations passing

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->