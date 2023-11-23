# Sablier V2 Services [![Discord][discord-badge]][discord] [![Twitter][twitter-badge]][twitter]

[discord]: https://discord.gg/bSwRCwWRsT
[discord-badge]: https://dcbadge.vercel.app/api/server/bSwRCwWRsT?style=flat
[twitter]: https://twitter.com/Sablier
[twitter-badge]: https://img.shields.io/twitter/follow/Sablier?label=%40Sablier

Web services that extend the functionality of the Sablier V2 token distribution protocol.

For more details about Sablier, check out our [website](https://sablier.com) and our
[documentation](https://docs.sablier.com).

## Usage

In the [`apps/merkle-api`](./apps/merkle-api/) directory, you will find an `.env.example` file that looks like this:

```text
PINATA_API_KEY=
PINATA_SECRET_API_KEY=
IPFS_GATEWAY=
```

In order to make the API work properly you will need to create a .env file at the same level with the .env.example.
After a campaign is validated and created we use Pinata to upload and in the file to IPFS. In order to obtain the
PINATA_API_KEY and PINATA_SECRET_API_KEY follow this steps:

1. Sign up or log in on https://app.pinata.cloud/
1. Select the API Keys tab
1. Select New Key
1. The key should have the permissions `pinFileToIPFS` and `pinJSONToIPFS`
1. Set a name for the key
1. Click Create Key
1. From the popup, take the API Key and the API Secret and put them in the `.env` file. The `IPFS_GATEWAY` variable can
   be any IPFS gateway but we recommend using a private one (Pinata offers this as well). For more details about the
   interactions with IPFS, check [`apps/merkle-api/src/services/ipfs.rs`](./apps/merkle-api/src/services/ipfs.rs).

We use Vercel for hosting, and this is why we have separate binaries for each endpoint. To run locally, you can use this
command:

```sh
$ vercel dev
```

Or, to simulate the cloud environment locally:

```sh
$ cd apps/merkle-api
$ cargo run --bin sablier_merkle_api
```

This command will run a standard web API and expose it on port 8000 on localhost. Even tough we are using Cargo
workspaces, the `cargo run` command should be run from inside the [`apps/merkle-api`](./apps/merkle-api) directory in
order to properly use the `.env` files.

### Local Development

The `.cargo/config.toml` file is used strictly for Vercel, so you can remove it during local development. However, if
you would prefer not to remove it and you happen to use macOS, configure your `config.toml` file like this:

```sh
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

Now you can start making changes.

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
$ cd apps/merkle-api
$ cargo run --bin sablier_merkle_api
```

To add another crate, you can use this command:

```sh
$ cd apps/merkle-api
$ cargo add crate_name
```

### CSV

You can an example of a Rust CSV Generator here:

https://gist.github.com/gavriliumircea/2a9797f207a2a2f3832ddaa376337e8c

## Contributing

Feel free to dive in! [Open](https://github.com/sablier-labs/v2-services/issues/new) an issue,
[start](https://github.com/sablier-labs/v2-services/discussions/new) a discussion or submit a PR.

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
