# Autometrics List

A command that lists all functions that have the "autometrics" annotation.

The aim is to use this binary as a quick static analyzer that returns from a
codebase the complete list of functions that are annotated to be
autometricized.

The analysis is powered by [Tree-sitter](https://tree-sitter.github.io), and
all the specific logic is contained in [Tree-sitter queries](./runtime/queries)
that are specific for each language implementation.

## Quickstart

Use the installer script to pull the latest version directly from Github
(change `VERSION` accordingly):

```console
VERSION=0.2.0 curl --proto '=https' --tlsv1.2 -LsSf https://github.com/gagbo/am_list/releases/download/v$VERSION/am_list-installer.sh | sh
```

And run the binary
```bash
# Make sure that `~/.cargo/bin` is in your `PATH`
am_list list -l rs /path/to/project/root
```

## Current state and known issues

### Language support table

In the following table, having the "detection" feature means that `am_list`
returns the exact same labels as the ones you would need to use in PromQL to
look at the metrics. In a nutshell,
"[Autometrics](https://github.com/autometrics-dev) compliance".

Language | Function name detection | Module detection
:---:|:---:|:---:
[Rust](https://github.com/autometrics-dev/autometrics-rs) | ✅ | ✅
[Typescript](https://github.com/autometrics-dev/autometrics-ts) | ❌ | ❌
[Go](https://github.com/autometrics-dev/autometrics-go) | ✅ | ✅
[Python](https://github.com/autometrics-dev/autometrics-py) | ❌ | ❌
[C#](https://github.com/autometrics-dev/autometrics-cs) | ❌ | ❌


### Rust

#### Aliasing issues

`am_list` doesn't track type renaming across files. That means for example that if
- you created a `struct Foo` in `src/foo.rs`,
- and then imported it as
```rust
use crate::foo::Foo as Oof;

#[autometrics]
impl Oof {
    // implOofBlock
}
```

then all the functions in the `implOofBlock` won't be detected by this utility
(_but would still work in autometrics_). This is not planned to be fixed, as it
might not even be legal in Rust, and at the very least is going to be very
rare.
