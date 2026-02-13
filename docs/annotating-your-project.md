# Annotating Your Project

A step-by-step guide for adopting archidoc on an existing codebase.

## Prerequisites

- archidoc installed (`cargo install archidoc-cli`)
- A Rust or TypeScript project with a directory-per-module structure

## Step 1: Identify Your Containers

Look at your project's top-level `src/` directories. Each directory that represents a major subsystem is likely a C4 **container**.

Good heuristics:
- Could this directory be deployed or versioned independently?
- Does it have its own tests or README?
- Would a new team member understand this as a distinct subsystem?

Example mapping:

```
src/
  api/          -> container (REST gateway)
  database/     -> container (persistence layer)
  events/       -> container (event bus)
  utils/        -> probably NOT a container (shared utilities)
```

Start with 3-5 containers. You can always add more later.

## Step 2: Add Your First Annotation

Open the entry file for your first container (`mod.rs` for Rust, `index.ts` for TypeScript) and add the annotation block.

**Rust** (`src/api/mod.rs`):

```rust
//! @c4 container
//!
//! # Api
//!
//! REST API gateway — handles authentication and request routing.
```

**TypeScript** (`src/api/index.ts`):

```typescript
/**
 * @c4 container
 *
 * REST API gateway — handles authentication and request routing.
 */
```

The key parts:
- **C4 marker**: `@c4 container` — tells archidoc this is a C4 container
- **Description**: First non-empty line after the marker — appears in diagrams and docs

## Step 3: Run archidoc

```bash
archidoc .
```

Output:

```
archidoc: 1 modules
wrote ARCHITECTURE.md
```

Open `ARCHITECTURE.md` — you should see a Mermaid C4 diagram with your container, a component index table linking to source files, and a relationship map.

## Step 4: Add Relationships

Declare dependencies between containers using relationship markers.

**Rust**:

```rust
//! @c4 container
//!
//! # Api
//!
//! REST API gateway — handles authentication and request routing.
//!
//! @c4 uses database "Persists user data" "sqlx"
//! @c4 uses events "Publishes domain events" "channel"
```

**TypeScript**:

```typescript
/**
 * @c4 container
 *
 * REST API gateway — handles authentication and request routing.
 *
 * @c4 uses database "Persists user data" "HTTP"
 * @c4 uses events "Publishes domain events" "WebSocket"
 */
```

Each relationship has three parts:
1. **Target**: the module path of the dependency (e.g., `database`)
2. **Label**: what data flows (e.g., `"Persists user data"`)
3. **Protocol**: how the communication happens (e.g., `"sqlx"`, `"HTTP"`)

## Step 5: Add File Tables

Document each module's internal structure with a file table.

```rust
//! @c4 container
//!
//! # Api
//!
//! REST API gateway — handles authentication and request routing.
//!
//! @c4 uses database "Persists user data" "sqlx"
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `routes.rs` | -- | HTTP route handlers | active |
//! | `middleware.rs` | Strategy | Auth and rate limiting | stable |
//! | `errors.rs` | -- | Error types and conversions | stable |
```

Column definitions:
- **File**: filename with backtick formatting
- **Pattern**: GoF design pattern name, or `--` for none
- **Purpose**: one-line description of the file's responsibility
- **Health**: `planned` (not yet implemented), `active` (in progress), `stable` (complete)

Tips:
- Skip structural files (`mod.rs`, `lib.rs`, `main.rs`) — archidoc ignores them
- Every `.rs`/`.ts` file in the directory should appear in the table
- Run `archidoc --validate .` to detect ghosts (listed but missing) and orphans (exist but not listed)

## Step 6: Add Components

Use `@c4 component` for sub-modules within a container. Components are nested inside their parent container in the C4 diagram.

```rust
//! @c4 component
//!
//! # Api.Auth
//!
//! Authentication and authorization middleware.
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `jwt.rs` | -- | JWT token validation | stable |
//! | `rbac.rs` | Strategy | Role-based access control | active |
```

The module path (`api.auth`) automatically nests this component under the `api` container.

## Step 7: Gate Your CI

Add `archidoc --check` to your CI pipeline to prevent architecture drift:

```bash
archidoc --check .
```

This exits non-zero if the generated docs would differ from what's on disk. If someone changes an annotation but forgets to regenerate, CI catches it.

Example GitHub Actions step:

```yaml
- name: Check architecture drift
  run: archidoc --check .
```

Or use the pre-commit hook in `hooks/pre-commit`.

## Common Mistakes

**Wrong C4 level**: Use `@c4 container` for top-level subsystems, `@c4 component` for sub-modules within a container. If everything is a container, your diagram loses the hierarchy.

**Orphan files**: Files exist on disk but aren't in the file table. Run `archidoc --validate .` to find them. Either add them to the table or move them.

**Ghost entries**: File table lists a file that doesn't exist. Usually means the file was renamed or deleted. Update the table.

**Stale descriptions**: The description line is the first non-marker paragraph. If you refactor a module's purpose, update this line too.

**Missing relationships**: If module A imports from module B, add a `@c4 uses b "..." "..."` marker. The diagram should show all runtime dependencies.

## What Good Annotations Look Like

### REST API Container

```rust
//! @c4 container
//!
//! # Api
//!
//! REST API gateway — handles authentication, rate limiting, and request routing.
//!
//! @c4 uses database "Persists user data" "sqlx"
//! @c4 uses events "Publishes domain events" "channel"
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `routes.rs` | -- | HTTP route handlers | active |
//! | `middleware.rs` | Strategy | Auth and rate limiting | stable |
//! | `errors.rs` | -- | Error types and conversions | stable |
```

### Database Adapter

```rust
//! @c4 container
//!
//! # Database
//!
//! Persistence layer — manages connection pooling and query execution.
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `pool.rs` | Singleton | Connection pool management | stable |
//! | `queries.rs` | Repository | SQL query implementations | active |
//! | `migrations.rs` | -- | Schema migration runner | planned |
```

### Event Bus

```rust
//! @c4 container
//!
//! # Events
//!
//! Domain event bus — decouples producers from consumers via channels.
//!
//! | File | Pattern | Purpose | Health |
//! |------|---------|---------|--------|
//! | `bus.rs` | Observer | Event dispatch and subscription | active |
//! | `types.rs` | -- | Event type definitions | stable |
```
