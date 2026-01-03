# {{plugin-name}}

{{description}}

## Building

```bash
cargo build --release
```

## Installation

Copy the compiled library to the editor's plugins directory:

- macOS: `target/release/lib{{plugin-name}}.dylib`
- Linux: `target/release/lib{{plugin-name}}.so`
- Windows: `target/release/{{plugin-name}}.dll`

## Development

### Running Tests

```bash
cargo test
```

### Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Author

{{author}}

## License

MIT
