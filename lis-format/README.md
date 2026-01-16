# LIS Format

A code formatter for the LIS (Language for Intelligent Systems) programming language.

## Features

- **AST-based formatting** - Ensures syntactic correctness
- **Configurable style** - Spaces vs tabs, alignment, spacing
- **Fast & reliable** - Written in Rust for performance
- **VSCode integration** - Format on save and on command
- **Multiple output modes** - Format in-place, check only, or output to stdout

## Installation

### From Source

```bash
cargo install --path lis-format
```

### In PATH

Make sure the `lis-format` binary is in your PATH for VSCode integration.

## Usage

### Command Line

```bash
# Format a file and print to stdout
lis-format file.lis

# Format a file in-place
lis-format -w file.lis

# Check if files are formatted (exit code 1 if not)
lis-format --check *.lis

# Format from stdin
cat file.lis | lis-format -

# Use compact style (minimal whitespace)
lis-format --compact file.lis

# Use readable style (extra whitespace)
lis-format --readable file.lis

# Custom indentation
lis-format --indent tabs file.lis
lis-format --indent spaces --indent-size 2 file.lis
```

### VSCode

The formatter is automatically integrated when you install the SIL & LIS extension.

**Format document**:

- `Shift+Alt+F` (Windows/Linux)
- `Shift+Option+F` (macOS)
- Or right-click → "Format Document"

**Enable format on save**:

```json
{
  "[lis]": {
    "editor.formatOnSave": true
  }
}
```

## Configuration

### Default Style

```
fn main() {
    let x = 42;
    let y = x + 10;
    return y;
}

fn process(input: State, threshold: ByteSil) {
    let processed = input |> normalize;
    if processed.L0 > threshold {
        act(processed);
    }
    return processed;
}
```

### Compact Style (`--compact`)

```
fn main(){
    let x=42;
    let y=x+10;
    return y;
}
fn process(input: State,threshold: ByteSil){
    let processed=input|>normalize;
    if processed.L0>threshold{
        act(processed);
    }
    return processed;
}
```

### Readable Style (`--readable`)

```
fn main() {
    let x = 42;
    let y = x + 10;
    return y;
}


fn process(input: State, threshold: ByteSil) {
    let processed = input |> normalize;
    if processed.L0 > threshold {
        act(processed);
    }
    return processed;
}
```

## Options

| Flag | Description | Default |
|------|-------------|---------|
| `-w, --write` | Write formatted output back to file(s) | false |
| `-c, --check` | Check if files are formatted (CI mode) | false |
| `--indent <TYPE>` | Indentation style: `spaces` or `tabs` | `spaces` |
| `--indent-size <N>` | Number of spaces per indent level | `4` |
| `--max-width <N>` | Maximum line width | `100` |
| `--no-align-assignments` | Disable alignment of assignments | enabled |
| `--no-space-operators` | Disable spaces around operators | enabled |
| `--compact` | Minimal whitespace preset | - |
| `--readable` | Extra whitespace preset | - |
| `-v, --verbose` | Print formatting statistics | false |

## VSCode Settings

```json
{
  // Path to lis-format binary
  "lis.formatter.path": "lis-format",

  // Enable formatting
  "lis.format.enable": true,

  // Format on save
  "[lis]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "sil-.sil-language"
  }
}
```

## CI Integration

Use `--check` mode in your CI pipeline:

```bash
# Check all LIS files
lis-format --check src/**/*.lis

# Exit code 0 = all formatted
# Exit code 1 = needs formatting
```

### GitHub Actions Example

```yaml
- name: Check LIS formatting
  run: |
    cargo install --path lis-format
    lis-format --check --verbose src/**/*.lis
```

## How It Works

1. **Lexical Analysis** - Source code is tokenized by `lis-core`
2. **Parsing** - Tokens are parsed into an AST (Abstract Syntax Tree)
3. **Pretty Printing** - AST is traversed and formatted according to style rules
4. **Output** - Formatted code is written to stdout or file

This approach ensures that only syntactically valid code is formatted, and the formatter never breaks working code.

## Formatting Rules

### Spacing

- Spaces around binary operators (`x + y`, not `x+y`)
- Space after comma (`fn(a, b)`, not `fn(a,b)`)
- Space before opening brace (`if x {`, not `if x{`)
- No space in unary operators (`-x`, not `- x`)

### Indentation

- One indentation level per nested block
- Consistent indentation throughout file
- Configurable spaces or tabs

### Alignment

- Function parameters on one line when possible
- State construction fields aligned
- Assignment operators can be aligned (configurable)

### Line Length

- Respects `--max-width` setting
- Breaks long lines intelligently
- Preserves readability

## Examples

### Before Formatting

```lis
fn main(){let x=42;let y=x+10;return y;}

fn process(input:State,threshold:ByteSil){
let processed=input|>normalize;
if processed.L0>threshold{
act(processed);
}
return processed;
}
```

### After Formatting

```lis
fn main() {
    let x = 42;
    let y = x + 10;
    return y;
}

fn process(input: State, threshold: ByteSil) {
    let processed = input |> normalize;
    if processed.L0 > threshold {
        act(processed);
    }
    return processed;
}
```

## Contributing

Contributions are welcome! Please ensure your changes:

1. Pass all tests: `cargo test -p lis-format`
2. Don't break existing formatting
3. Include tests for new features
4. Follow Rust style guidelines

## License

AGPL-3.0 - See LICENSE file for details
