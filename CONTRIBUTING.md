# Contributing to Yetlet

Yetlet is a maintainer-directed project. The upstream repository does **not** accept external pull requests.

## What is welcome

External users may open GitHub Issues for:

- reproducible bug reports;
- compatibility reports with Rust, browsers or operating systems;
- documentation feedback;
- focused feature proposals that preserve Yetlet's small, HTML-first, handler-first and opt-in design.

Please include the Yetlet commit/version, Rust version, relevant OS/browser versions, a minimal reproduction, and expected versus observed behavior.

The maintainer may accept an idea and implement it independently. Opening an Issue does not grant the issue author merge or release authority.

## Pull requests and forks

Pull request creation is restricted to repository collaborators. Maintainers may use pull requests for internal review and required CI checks, but unsolicited external pull requests are not accepted.

Forking and independent modification are permitted under the project license. A fork is not an upstream contribution channel.

Because external code contributions are not accepted, Yetlet does not require a Contributor License Agreement, Developer Certificate of Origin, or `Signed-off-by` line.

## Security reports

Do not report a suspected vulnerability through a public Issue. Follow [SECURITY.md](SECURITY.md) and use the private reporting channel available for the repository.

## Developer entry points

Start with the [native SSR quickstart](README.md#start-with-native-ssr),
[documentation index](docs/README.md) and [sample guide](docs/SAMPLES.md).
The Experimental `v0.9.0-alpha.1` source-package preview is
[published on GitHub](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.1);
crates.io publication remains separate. Retained evidence applies to its exact
source and scope, not every later checkout.

Authorized maintainers and their agents should use the
[development/PR SOP](docs/DEVELOPMENT.md) and [release SOP](docs/RELEASING.md),
with [repository instructions](AGENTS.md) for bounded changes and authority.
These links do not change the external contribution policy above.
