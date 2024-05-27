[![cargo_ci](https://github.com/kv5h/session-manager-wrapper/actions/workflows/cargo_ci.yaml/badge.svg)](https://github.com/kv5h/session-manager-wrapper/blob/main/.github/workflows/cargo_ci.yaml)
![rust_version](https://img.shields.io/badge/Rust_version-rustc_1.80.0--nightly-red)

# session-manager-wrapper

A Rust wrapper for AWS SSM Session Manager

## Usage

```
Usage: session-manager-wrapper [OPTIONS] --instance-id <instance id>

Options:
  -i, --instance-id <instance id>  Instance ID
  -l, --local-port <local port>    Local port
                                   If `0` is specified, an arbitrary free port will be assigned.
  -p, --remote-port <remote port>  Remote port
  -r, --remote-host <remote host>  Remote host
                                   Required only for port forwarding over bastion server
  -h, --help                       Print help
  -V, --version                    Print version
```
