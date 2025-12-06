# CrawlStack

[![CI Tests](https://github.com/Rich-T-kid/CrawlStack/actions/workflows/test.yml/badge.svg)](https://github.com/Rich-T-kid/CrawlStack/actions/workflows/test.yml)
[![Rust Lint](https://github.com/Rich-T-kid/CrawlStack/actions/workflows/lint.yml/badge.svg)](https://github.com/Rich-T-kid/CrawlStack/actions/workflows/lint.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

An inverted web scraping index written in Rust.

## Overview

CrawlStack is a high-performance web scraping index implemented in Rust. It provides an efficient way to organize, search, and retrieve scraped web data using an inverted index structure.

## Features

- **Fast Indexing**: Built with Rust for maximum performance and memory safety
- **Inverted Index**: Efficient data structure for quick lookups and searches
- **Memory Safe**: Leverages Rust's ownership system to prevent common bugs
- **Concurrent Processing**: Built to handle multiple scraping operations efficiently

## Installation

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/Rich-T-kid/CrawlStack.git
cd CrawlStack

# Build the project
cargo build --release

# Run tests
cargo test
```

## Development

### Running Tests

```bash
cargo test
```

### Running Linter

```bash
cargo clippy -- -D warnings
```

### Building Documentation

```bash
cargo doc --open
```


## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

