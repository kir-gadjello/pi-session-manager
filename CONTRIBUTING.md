# Contributing to Pi Session Manager

Thank you for your interest in contributing to Pi Session Manager!

## Getting Started

### Prerequisites

- Node.js 18+
- Rust 1.70+
- Tauri CLI

### Setup

1. Clone the repository:
```bash
git clone <repository-url>
cd pi-session-manager
```

2. Install dependencies:
```bash
npm install
```

3. Run the development server:
```bash
npm run tauri:dev
```

## Project Structure

```
pi-session-manager/
├── src/                    # Frontend (React + TypeScript)
│   ├── components/         # React components
│   ├── hooks/             # Custom React hooks
│   ├── App.tsx            # Main app component
│   └── types.ts           # TypeScript type definitions
├── src-tauri/             # Backend (Rust)
│   ├── src/
│   │   ├── commands.rs    # Tauri IPC commands
│   │   ├── scanner.rs     # Session scanning
│   │   ├── search.rs      # Search functionality
│   │   ├── export.rs      # Export functionality
│   │   ├── stats.rs       # Statistics calculation
│   │   ├── models.rs      # Data models
│   │   └── lib.rs         # Library entry point
│   └── Cargo.toml         # Rust dependencies
└── tauri.conf.json        # Tauri configuration
```

## Development Workflow

### Running the App

```bash
# Development mode
npm run tauri:dev

# Build for production
npm run tauri:build
```

### Code Style

#### TypeScript/React
- Use functional components with hooks
- Follow React best practices
- Use TypeScript for type safety
- Keep components small and focused

#### Rust
- Follow Rust naming conventions
- Use `Result<T, String>` for error handling
- Document public functions
- Keep functions pure when possible

### Adding New Features

1. **Define the feature**: Create an issue or discussion first
2. **Design the API**: Think about both frontend and backend
3. **Implement backend**: Add Tauri commands in `commands.rs`
4. **Implement frontend**: Create React components
5. **Test thoroughly**: Test all edge cases
6. **Update documentation**: Update README and CHANGELOG

### Example: Adding a New Tauri Command

1. Add the function in `src-tauri/src/commands.rs`:
```rust
#[tauri::command]
pub async fn my_new_command(param: String) -> Result<String, String> {
    // Your logic here
    Ok("result".to_string())
}
```

2. Register the command in `src-tauri/src/lib.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    my_new_command
])
```

3. Call from frontend:
```typescript
const result = await invoke<string>('my_new_command', { param: 'value' });
```

## Testing

### Manual Testing Checklist

- [ ] Session list loads correctly
- [ ] Search works with various queries
- [ ] Export produces valid output
- [ ] Delete removes session from list
- [ ] Rename updates session name
- [ ] Statistics display correctly
- [ ] Keyboard shortcuts work
- [ ] UI is responsive

### Adding Tests

```bash
# Run frontend tests (when added)
npm test

# Run Rust tests (when added)
cargo test
```

## Reporting Issues

When reporting issues, please include:

1. **OS and version**: e.g., macOS 14.0, Windows 11
2. **App version**: Check in app or package.json
3. **Steps to reproduce**: Detailed steps to reproduce the issue
4. **Expected behavior**: What you expected to happen
5. **Actual behavior**: What actually happened
6. **Logs**: Any relevant logs or error messages

## Submitting Changes

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Commit your changes: `git commit -am 'Add some feature'`
4. Push to the branch: `git push origin feature/my-feature`
5. Create a Pull Request

### Commit Message Format

Use conventional commit messages:

```
feat: add session export feature
fix: resolve search not returning results
docs: update README with new features
style: format code with prettier
refactor: simplify search logic
test: add unit tests for scanner
chore: update dependencies
```

## Code Review

All submissions require review before merging. We will:

- Review your code for quality and consistency
- Test your changes
- Provide feedback and suggest improvements
- Merge once approved

## Questions?

Feel free to open an issue for questions, suggestions, or discussions.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.