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
PINATA_ACCESS_TOKEN=
PINATA_API_KEY=
PINATA_API_SERVER=
PINATA_SECRET_API_KEY=
IPFS_GATEWAY=
```

After a campaign is created via the API, we use Pinata to upload and pin the file to IPFS. In order to obtain the
`PINATA_API_KEY`, `PINATA_SECRET_API_KEY` and `PINATA_ACCESS_TOKEN`, follow these steps:

1. Sign up or log in on https://app.pinata.cloud/
1. Select the API Keys tab
1. Select New Key
1. The key should have the permissions `pinFileToIPFS` and `pinJSONToIPFS`
1. Set a name for the key
1. Click Create Key
1. From the popup, take the API Key and the API Secret and put them in the `.env` file. The `IPFS_GATEWAY` variable can
   be any IPFS gateway but we recommend using a private one (Pinata offers this as well). For more details about the
   interactions with IPFS, check [`src/services/ipfs.rs`](./src/services/ipfs.rs).
1. Select the "Access Controls" tab
1. Click on the "Request Token" button
1. Copy the token and put in th `.env` file in the `PINATA_ACCESS_TOKEN` variable

We use Vercel for hosting, and this is why we have separate binaries for each endpoint. For local development, use this
command:

```sh
$ vercel dev
```

Or, to simulate the cloud environment:

```sh
$ cargo run --bin sablier_merkle_api
```

Either of these commands will run a standard web API and expose it on port 3000 on localhost.

## API

The endpoints below assume that you are running the API locally.

Do not add trailing slashes to the API endpoints.

### Create

```text
POST http://localhost:3000/api/create?decimals=... + FORM_DATA{file: "a csv file with addresses and amounts"}
```

You can find a csv template here: `https://files.sablier.com/templates/campaignTemplate.csv`. Please use valid addresses
and positive amounts. The amount paddings will be performed based on the decimals query param.

### Eligibility

```text
GET http://localhost:3000/api/eligibility?address=...&cid=...
```

### Health

```text
GET http://localhost:3000/api/health
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

Sablier V2 Merkle API is licensed under [GPL v3 or later](./LICENSE.md).
