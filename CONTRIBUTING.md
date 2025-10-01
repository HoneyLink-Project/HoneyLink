# Contributing to HoneyLink™

Thank you for your interest in contributing to HoneyLink™! This document provides guidelines and instructions for contributing to the project.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Commit Message Guidelines](#commit-message-guidelines)
- [Pull Request Process](#pull-request-process)
- [Documentation Requirements](#documentation-requirements)
- [Testing Requirements](#testing-requirements)
- [Review Process](#review-process)

---

## 🤝 Code of Conduct

This project adheres to a professional code of conduct. By participating, you are expected to uphold this standard. Please be respectful and constructive in all interactions.

---

## 🚀 Getting Started

### Prerequisites

- **Rust**: 1.89.0+ (see [`docs/RUST_SETUP.md`](./docs/RUST_SETUP.md))
- **Node.js**: 22.15.0+ (see [`docs/NODE_SETUP.md`](./docs/NODE_SETUP.md))
- **pnpm**: 10.x+
- **Git**: 2.40+

### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/HoneyLink-Project/HoneyLink.git
cd HoneyLink

# Install Rust tools
cargo install cargo-llvm-cov cargo-audit cargo-deny

# Install UI dependencies
cd ui
pnpm install

# Run native dependency check
node ../scripts/audit-native-deps.js
```

### IDE Setup

Recommended IDEs:
- **VS Code** (推奨): Install recommended extensions from `.vscode/extensions.json`
- **IntelliJ IDEA / CLion**: Install Rust and TypeScript plugins

---

## 🔄 Development Workflow

### 1. Create an Issue

Before starting work, create or find an issue that describes the problem or feature. This helps coordinate work and avoid duplication.

### 2. Create a Feature Branch

```bash
git checkout -b <type>/<short-description>
```

Branch naming conventions:
- `feat/add-session-timeout` - New features
- `fix/session-leak` - Bug fixes
- `docs/api-reference` - Documentation
- `refactor/crypto-module` - Code refactoring
- `test/integration-suite` - Test additions
- `chore/update-deps` - Maintenance tasks

### 3. Make Changes

- Write clean, idiomatic code
- Follow project coding standards (see below)
- Add tests for new functionality
- Update documentation as needed
- Run linters and formatters before committing

### 4. Commit Your Changes

```bash
git add <files>
git commit -m "<type>(<scope>): <description>"
```

See [Commit Message Guidelines](#commit-message-guidelines) below.

### 5. Push and Create Pull Request

```bash
git push origin <your-branch>
```

Then create a Pull Request on GitHub.

---

## 📐 Coding Standards

### Rust Code Style

#### Formatting
- **Indentation**: 4 spaces
- **Line length**: Max 100 characters
- **Format**: Use `cargo fmt` (rustfmt)

#### Naming Conventions
- **Types/Structs/Enums**: `PascalCase`
- **Functions/Variables**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Lifetimes**: `'a`, `'b`, etc. (single lowercase letter)

#### Best Practices
- Use `Result<T>` for fallible operations
- Prefer `impl Trait` over explicit type parameters when appropriate
- Document public APIs with `///` doc comments
- Use `thiserror` for custom error types
- Avoid `unwrap()` and `expect()` in production code (use proper error handling)
- Use `clippy` recommendations (run `cargo clippy`)

#### Example
```rust
/// Session state machine for managing session lifecycle.
pub struct SessionStateMachine {
    session_id: SessionId,
    state: SessionState,
}

impl SessionStateMachine {
    /// Creates a new session state machine.
    pub fn new(session_id: SessionId) -> Self {
        Self {
            session_id,
            state: SessionState::Pending,
        }
    }

    /// Transitions to a new state.
    pub fn transition(&mut self, new_state: SessionState) -> Result<()> {
        // Validation logic
        self.state = new_state;
        Ok(())
    }
}
```

### TypeScript Code Style

#### Formatting
- **Indentation**: 2 spaces
- **Line length**: Max 100 characters
- **Quotes**: Single quotes for strings
- **Semicolons**: Required
- **Format**: Use Prettier

#### Naming Conventions
- **Types/Interfaces/Classes**: `PascalCase`
- **Functions/Variables**: `camelCase`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **React Components**: `PascalCase`

#### Best Practices
- Use TypeScript strict mode
- Prefer `const` over `let`
- Use arrow functions for callbacks
- Destructure props in React components
- Use explicit return types for functions
- Avoid `any` type (use `unknown` if necessary)
- Use ESLint recommendations (run `pnpm lint`)

#### Example
```typescript
interface SessionProps {
  sessionId: string;
  onClose: () => void;
}

export const SessionCard: React.FC<SessionProps> = ({ sessionId, onClose }) => {
  const [isLoading, setIsLoading] = useState(false);

  const handleClose = useCallback((): void => {
    setIsLoading(true);
    onClose();
  }, [onClose]);

  return (
    <div className="session-card">
      <h3>{sessionId}</h3>
      <button onClick={handleClose} disabled={isLoading}>
        Close
      </button>
    </div>
  );
};
```

### General Principles

1. **C/C++ Exclusion Policy**: Absolutely no C/C++ dependencies
2. **Security First**: Follow Zero Trust principles
3. **Memory Safety**: Leverage Rust's ownership system
4. **Error Handling**: Always handle errors gracefully
5. **Observability**: Add telemetry for production monitoring
6. **Testability**: Write testable, modular code
7. **Documentation**: Code should be self-documenting with clear comments

---

## 📝 Commit Message Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/) specification.

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, no logic change)
- `refactor`: Code refactoring (no功能 change)
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Build process or auxiliary tool changes
- `ci`: CI/CD configuration changes
- `revert`: Revert a previous commit

### Scope

The scope should indicate the module affected:
- `session`: Session Orchestrator
- `policy`: Policy Engine
- `crypto`: Crypto & Trust Anchor
- `transport`: Transport Layer
- `qos`: QoS Scheduler
- `telemetry`: Telemetry & Insights
- `adapter`: Physical Adapter
- `experience`: Experience Layer (SDK/UI)
- `ui`: UI-specific changes
- `docs`: Documentation
- `ci`: CI/CD

### Examples

```
feat(session): add idempotency-key support for session creation

- Implement UUID-based idempotency key validation
- Add 5-minute TTL for key storage
- Update session creation API to accept idempotency header

Refs: #123
```

```
fix(crypto): prevent key leakage in error messages

Use zeroize to clear sensitive data from memory before
returning error messages.

Fixes: #456
```

```
docs(api): update control plane API documentation

- Add examples for mTLS configuration
- Document OAuth2 token refresh flow

Refs: #789
```

---

## 🔀 Pull Request Process

### Before Submitting

1. **Run all tests**:
   ```bash
   cargo test --workspace
   cd ui && pnpm test
   ```

2. **Run linters**:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets --all-features
   cd ui && pnpm lint
   ```

3. **Check coverage** (optional but recommended):
   ```bash
   cargo llvm-cov --workspace --html
   ```

4. **Update documentation**:
   - Update `README.md` if adding new features
   - Update module-specific docs in `spec/modules/`
   - Add inline code comments for complex logic

### PR Template

When creating a PR, include:

**Title**: Follow commit message format (`type(scope): description`)

**Description**:
```markdown
## Changes
- Describe what changed

## Motivation
- Why was this change needed?

## Testing
- How was this tested?

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] No C/C++ dependencies introduced
- [ ] Linters pass
- [ ] Coverage maintained (>90% for critical modules)

## Related Issues
Closes #123
Refs #456
```

### Review Criteria

Your PR will be reviewed for:

1. **Correctness**: Does it solve the problem?
2. **Code Quality**: Is it clean, readable, and maintainable?
3. **Test Coverage**: Are there adequate tests?
4. **Documentation**: Is it properly documented?
5. **Performance**: Are there any performance concerns?
6. **Security**: Does it introduce security risks?
7. **Dependencies**: No C/C++ dependencies added?

---

## 📚 Documentation Requirements

### Code Documentation

#### Rust
- Public APIs must have `///` doc comments
- Include examples in doc comments when helpful
- Document panics, errors, and safety considerations

```rust
/// Derives a session key from the parent key.
///
/// # Arguments
///
/// * `parent_key` - The parent key for derivation
/// * `context` - Additional context for HKDF
///
/// # Returns
///
/// A 32-byte session key wrapped in `Zeroizing` for secure memory handling.
///
/// # Errors
///
/// Returns `Error::Crypto` if key derivation fails.
///
/// # Examples
///
/// ```
/// let parent = vec![0u8; 32];
/// let key = derive_session_key(&parent, b"session_123")?;
/// ```
pub fn derive_session_key(parent_key: &[u8], context: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    // Implementation
}
```

#### TypeScript
- Use TSDoc comments for public APIs
- Include `@param`, `@returns`, `@throws` tags

```typescript
/**
 * Fetches session details from the API.
 *
 * @param sessionId - The session identifier
 * @returns Promise resolving to session data
 * @throws {ApiError} If the session is not found
 *
 * @example
 * ```typescript
 * const session = await fetchSession('session-123');
 * console.log(session.state);
 * ```
 */
export async function fetchSession(sessionId: string): Promise<Session> {
  // Implementation
}
```

### Module Documentation

When adding a new module:

1. Create a document in `spec/modules/<module-name>.md` using [`spec/templates/module-template.md`](./spec/templates/module-template.md)
2. Define:
   - Responsibilities
   - Input/Output interfaces
   - State machines (if applicable)
   - Dependencies
   - Traceability to requirements

---

## 🧪 Testing Requirements

### Unit Tests

- **Coverage**: ≥90% for critical modules (crypto, session, policy)
- **Location**: Tests live in `#[cfg(test)]` modules or `tests/` directory
- **Naming**: Test functions use `snake_case` and descriptive names

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_transition_valid() {
        let mut sm = SessionStateMachine::new(SessionId::new());
        assert_eq!(sm.state(), SessionState::Pending);
        
        sm.transition(SessionState::Active).unwrap();
        assert_eq!(sm.state(), SessionState::Active);
    }

    #[test]
    fn test_session_state_transition_invalid() {
        let mut sm = SessionStateMachine::new(SessionId::new());
        let result = sm.transition(SessionState::Closed);
        assert!(result.is_err());
    }
}
```

### Integration Tests

- Follow scenarios in [`spec/testing/integration-tests.md`](./spec/testing/integration-tests.md)
- Use realistic data and multi-component interactions

### Property-Based Tests

- Use `proptest` for critical modules
- Test invariants and edge cases

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_key_derivation_deterministic(seed in any::<Vec<u8>>()) {
        let key1 = derive_key(&seed, b"context");
        let key2 = derive_key(&seed, b"context");
        assert_eq!(key1, key2);
    }
}
```

---

## 👀 Review Process

### Timeline

- **Initial Response**: Within 1 business day
- **First Review**: Within 2 business days
- **Follow-up Reviews**: Within 1 business day

### Approval Requirements

- **Minimum**: 1 approval from code owner
- **Critical Modules** (crypto, session): 2 approvals
- **Architecture Changes**: Architecture WG approval

### Merge Strategy

- **Squash and Merge**: For feature branches (default)
- **Rebase and Merge**: For clean, atomic commits
- **Merge Commit**: For cross-module integrations

---

## 🆘 Getting Help

- **Issues**: Check existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions
- **Spec Documents**: Refer to `spec/` directory for detailed documentation
- **Code Owners**: See `CODEOWNERS` file for module experts

---

## 📄 License

By contributing to HoneyLink™, you agree that your contributions will be licensed under the project's proprietary license.

---

**Thank you for contributing to HoneyLink™! 🍯**
