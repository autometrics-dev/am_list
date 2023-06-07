# Contributing

## Release management

The complete release cycle is handled and automated by the combination of `cargo-dist` and `cargo-release`.

When ready for a release, using `cargo release --(patch|minor|major)` should be enough.

### Release a breaking `0.Y` version

- Create a branch `release/0.Y` from main
- Run `cargo release 0.Y.0` and make sure everything is okay

<details>
<summary>If everything went ok</summary>
- Run the same command, but with `--execute` flag
</details>

<details>
<summary>If something went wrong</summary>
- Nothing got actually published, so you can reset the `release/0.Y` branch to `main`,
- Then make and commit the changes on your `release/0.Y` branch (that is still local),
- Then try the `cargo release 0.Y.0` again.
</details>
