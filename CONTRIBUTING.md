# Contributing

We welcome contributions! Here's how you can get involved:

## Getting Started

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'feat: add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Project Guidelines

- Use Rust with strict compiler checks (`#![deny(warnings)]`)
- Follow the existing code style
- Add doc comments for public items
- Run clippy before committing (`cargo clippy`)
- Write clear, descriptive commit messages, use [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) style

## Testing

Before submitting a pull request, please ensure that:

1. The project builds without errors (`cargo build`)
2. You run clippy and fix any issues (`cargo clippy`)
3. All tests pass (`cargo test`)
4. The code follows the project guidelines
5. New functionality includes appropriate tests

## Development Setup

### Prerequisites

- Rust (edition 2024)
- Git

### Installation

1. Clone the repository:
    ```bash
    git clone https://github.com/rom100main/txtodo.git
    cd txtodo
    ```

2. Build the project:
    ```bash
    cargo build
    ```

### Development Workflow

1. Make changes to the source code in the `src/` directory.

2. Run tests:
    ```bash
    cargo test
    ```

3. Check for lints:
    ```bash
    cargo clippy
    ```

## Project Structure

```
txtodo/
├── src/                      # Source code
│   ├── lib.rs                # Public exports
│   ├── todotxt.rs            # Main TodoTxt struct
│   ├── task.rs               # Task struct and utilities
│   ├── parser.rs             # Todo.txt parser
│   ├── serializer.rs         # Todo.txt serializer
│   ├── extension.rs          # Extension handling
│   ├── filters.rs            # Task filtering functions
│   ├── sorts.rs              # Task sorting functions
│   ├── options.rs            # Configuration options
│   ├── error.rs              # Error types
│   └── date_utils.rs         # Date parsing utilities
├── tests/                    # Integration tests
├── Cargo.toml                # Dependencies and metadata
└── README.md                 # Project documentation
```

## Architecture

### Core Components

1. **TodoTxt** (`src/todotxt.rs`): Main struct that provides the high-level API for parsing, serializing, and managing todo.txt files.

2. **TodoTxtParser** (`src/parser.rs`): Handles parsing of todo.txt format into Task objects.

3. **TodoTxtSerializer** (`src/serializer.rs`): Converts Task objects back to todo.txt format.

4. **ExtensionHandler** (`src/extension.rs`): Manages custom key:value extensions with automatic parsing and serialization.

## License

By contributing to this project, you agree that your contributions will be licensed under the MIT License.
