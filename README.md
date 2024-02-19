# Sablier V2 Merkle API [![Discord][discord-badge]][discord] [![Twitter][twitter-badge]][twitter]

[discord]: https://discord.gg/bSwRCwWRsT
[discord-badge]: https://dcbadge.vercel.app/api/server/bSwRCwWRsT?style=flat
[twitter]: https://twitter.com/Sablier
[twitter-badge]: https://img.shields.io/twitter/follow/Sablier?label=%40Sablier

A web API for generating and verifying Merkle trees used in Sablier V2.

For more details about Sablier, check out our [website](https://sablier.com) and our
[documentation](https://docs.sablier.com/api/merkle-api/intro).

## Development and usage

To integrate the Sablier V2 Merkle API into your own product or if you want ro test locally please check
[here](https://docs.sablier.com/api/merkle-api/development).

## API

The API provides endpoints that support actions like: creating a campaign, checking eligibility for a particular address
etc. For more details see [endpoints](https://docs.sablier.com/api/merkle-api/functionality)

### CSV

You can an example of a Rust CSV Generator here:

https://gist.github.com/gavriliumircea/2a9797f207a2a2f3832ddaa376337e8c

## Contributing

Feel free to dive in! [Open](https://github.com/sablier-labs/v2-merkle-api/issues/new) an issue,
[start](https://github.com/sablier-labs/v2-merkle-api/discussions/new) a discussion or submit a PR.

### Pre Requisites

You will need the following software on your machine:

- [Git](https://git-scm.com/downloads)
- [Rust](https://rust-lang.org/tools/install)
- [Cargo](https://doc.rust-lang.org/cargo/commands/cargo-install.html)

### Syntax Highlighting

You will need the following VSCode extensions:

- [rust-syntax](https://marketplace.visualstudio.com/items?itemName=dustypomerleau.rust-syntax)
- [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode)

## Recommendations

We recommend forking this repository and running the merkle backend using your own infrastructure or a vercel
environment hosted under an account you own. This guarantees you'll have more control over the up-time of the service,
as well as access to add any custom features or optimisations you may require.

## License

Sablier V2 Merkle API is licensed under [GPL v3 or later](./LICENSE.md).
