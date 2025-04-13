# SSHWrap

This tool provides a wrapper for SSH connections based on configurable patterns in a TOML file.

## Usage

```bash
sshwrap [--debug] <host> [ssh-args...]
```

- `--debug`: Enables debug output.
- `<host>`: Target host name that may match one of the patterns.
- `[ssh-args...]`: Additional arguments passed to the `ssh` command.

## Configuration

Patterns are stored in `.ssh/wrapper.toml` using an array of tables. Each pattern has its own `pattern` (used as a regex) and `add` (used to build the final host).

Example (TOML):
```toml
[[patterns]]
pattern = "exmpl-(.*)-sbx-(.*)"
add = "ubuntu@{1}.sbx.example.com"

[[patterns]]
pattern = "(.*).sbx"
add = "ubuntu@{1}.sbx.example.com"