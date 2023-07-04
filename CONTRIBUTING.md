# Contributing

## Release management

The complete release cycle is handled and automated by the combination of `cargo-dist` and `cargo-release`.

When ready for a release, using `cargo release --(patch|minor)` or `cargo release NEW_MAJOR` should be enough.

### Prerequisites

Only `cargo-release` is needed to create a new release:
```console
$ cargo install --locked cargo-release
```

Also, you need to have the authorization to both:
- create and push tags on `release/*` branches in the Github repo
- create and push crates on `crates-io` registry for [`am_list` crate](https://crates.io/crates/am_list)

### Release a breaking `0.Y` version

- Create a branch `release/0.Y` from main
- Run `cargo release 0.Y.0` and make sure everything is okay

**If everything went ok**

- Run the same command, but with `--execute` flag

**If something went wrong**

- Nothing got actually published, so you can reset the `release/0.Y` branch to `main`,
- Then make and commit the changes on your `release/0.Y` branch (that is still local),
- Then try the `cargo release 0.Y.0` again.

### Release a new version on a release branch

In the example, we will push a new `0.2.3` version on `0.2`

- Create a release branch `rel_0.2.3` with all the changes that need to be included
  + new features are cherry-picked from `main`
  + bugfixes happen directly on the release branch (and later get cherry-picked _to_ `main` if relevant)
- Push the new branch `rel_0.2.3` and create a PR to `release/0.2`
- Prepare the release: `cargo release --no-publish --no-tag --allow-branch=rel_0.2.3 patch`
- Cleanup the `CHANGELOG.md` that got bad replacement patterns because of the 2-step process
- Push the new commits, and merge the PR
- Switch locally to `release/0.2` (the "main" release branch), and pull the latest changes
- Publish the tag and the new crate: `cargo release publish --execute && cargo release tag --execute && cargo release push --execute`

Bonus:
- Merge the CHANGELOG/README/CONTRIBUTING back from `release/0.2` to `main`
