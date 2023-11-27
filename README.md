# Sablier V2 Merkle API [![Discord][discord-badge]][discord] [![Twitter][twitter-badge]][twitter]

[discord]: https://discord.gg/bSwRCwWRsT
[discord-badge]: https://dcbadge.vercel.app/api/server/bSwRCwWRsT?style=flat
[twitter]: https://twitter.com/Sablier
[twitter-badge]: https://img.shields.io/twitter/follow/Sablier?label=%40Sablier

A web API for generating and verifying Merkle trees used in Sablier V2.

For more details about Sablier, check out our [website](https://sablier.com) and our
[documentation](https://docs.sablier.com).

## Usage

In order to make the API work properly, you will need to create a `.env` file by following the `.env.example` file:

```text
PINATA_API_KEY=
PINATA_SECRET_API_KEY=
IPFS_GATEWAY=
```

After a campaign is created via the API, we use Pinata to upload and pin the file to IPFS. In order to obtain the
`PINATA_API_KEY` and `PINATA_SECRET_API_KEY`, follow this steps:

1. Sign up or log in on https://app.pinata.cloud/
1. Select the API Keys tab
1. Select New Key
1. The key should have the permissions `pinFileToIPFS` and `pinJSONToIPFS`
1. Set a name for the key
1. Click Create Key
1. From the popup, take the API Key and the API Secret and put them in the `.env` file. The `IPFS_GATEWAY` variable can
   be any IPFS gateway but we recommend using a private one (Pinata offers this as well). For more details about the
   interactions with IPFS, check [`src/services/ipfs.rs`](./src/services/ipfs.rs).

We use Vercel for hosting, and this is why we have separate binaries for each endpoint. To run locally, use this
command:

```sh
$ vercel dev
```

Or, to simulate the cloud environment locally:

```sh
$ cargo run --bin sablier_merkle_api
```

This command will run a standard web API and expose it on port 8000 on localhost.

### Local Development

The `.cargo/config.toml` file is used strictly for Vercel, so you can remove it during local development. However, if
you would prefer not to remove it and you happen to use macOS, configure your `config.toml` file like this:

```toml
[build]
   #target = "x86_64-unknown-linux-gnu"

[target.x86_64-unknown-linux-musl]
   linker = "x86_64-unknown-linux-gnu-gcc"
```

Alternatively, on Linux:

```sh
[build]
   target = "x86_64-unknown-linux-gnu"
```

## API

Do not add trailing slashes to the API endpoints.

### Create

```text
POST http://localhost:8000/api/create?decimals=... + FORM_DATA{file: "a csv file with addresses and amounts"}
```

### Eligibility

```text
GET http://localhost:8000/api/eligibility?address=...&cid=...
```

### Health

```text
GET http://localhost:8000/api/health
```

## Commands

Here's a list of the most frequently needed commands.

### App

To build the app, you can use this command:

```sh
$ cargo build
```

To run the app locally, you can use this command:

```sh
$ cargo run --bin sablier_merkle_api
```

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

## License

Sablier V2 Services is licensed under [GPL v3 or later](./LICENSE.md).
