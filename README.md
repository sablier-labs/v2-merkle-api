# Sablier V2 Services [![Styled with Prettier][prettier-badge]][prettier] [![Sablier][twitter-badge]][twitter]

[prettier]: https://prettier.io
[prettier-badge]: https://img.shields.io/badge/Code_Style-Prettier-ff69b4.svg
[twitter]: https://twitter.com/Sablier
[twitter-badge]: https://img.shields.io/twitter/follow/Sablier?label=%40Sablier

The official Sablier V2 backend services.

## Contributing

Feel free to dive in! [Open](https://github.com/sablier-labs/v2-services/issues/new) an issue,
[start](https://github.com/sablier-labs/v2-services/discussions/new) a discussion or submit a PR.

### Pre Requisites

You will need the following software on your machine:

- [Git](https://git-scm.com/downloads)
- [Rust](https://www.rust-lang.org/tools/install)
- [Cargo](https://doc.rust-lang.org/cargo/commands/cargo-install.html)

### Set Up

Clone this repository. In the app/merkle-api directory you will find the .env.example file that contains 3 environment variables:

```sh
PINATA_API_KEI=
PINATA_SECRET_API_KEY=
IPFS_GATEWAY=
```

In order to make the API work properly you will need to create a .env file at the same level with the .env.example. After a campaign is validated and created we use Pinata to upload and in the file to IPFS. In order to obtain the PINATA_API_KEY and PINATA_SECRET_API_KEY follow this steps:
1.  https://app.pinata.cloud/
2.  Log In/ Sign Up
3.  Select the API Keys tab
4.  Select New Key
5.  The key should have the permissions: pinFileToIPFS, pinJSONToIPFS
6.  Set a name for the key
7.  Click Create Key
8.  From the popup you can take the API Key and the API Secrete and fill .env with those values
The IPFS_GATEWAY= var can be any correct ipfs gateway but we recommend using a private one(Pinata offers this as well).
For more details about the interactions with ipfs check apps/merkle-api/src/services/ipfs.rs

We are using Vercel as hosting environment and that is why we have separate bins for each endpoint. In order to run locally you can either use:
```sh
$ vercel dev
```
to simulate locally what is happening in the cloud env or:
```sh
$ cd apps/merkle-api
$ cargo run --bin sablier_merkle_api
```
that will run a classic WEB API using the src/main.rs file.
Even tough we are using cargo work spaces the cargo run command should be run from inside the app/merkle-api in order to properly use the .env files.

The .cargo/config.toml is strictly used for vercel you can remove it for local development. If you don't want to remove it and you are using MacOs the config.toml should look like this:
```sh
[build]
#target = "x86_64-unknown-linux-gnu"

[target.x86_64-unknown-linux-musl]
linker = "x86_64-unknown-linux-gnu-gcc"
```
If you are using a linux distribution:
```sh
[build]
target = "x86_64-unknown-linux-gnu"
```

Now you can start making changes.

### Syntax Highlighting

You will need the following VSCode extensions:

- [Rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
- [Prettier](https://marketplace.visualstudio.com/items?itemName=esbenp.prettier-vscode)
- [Rust syntax](https://marketplace.visualstudio.com/items?itemName=dustypomerleau.rust-syntax)

## Commands

Here's a list of the most frequently needed commands.

### App

To build the app, you can run this command:

```sh
$ cargo build
```

To start the app on localhost, you can run this command:

```sh
$ cd apps/merkle-api
$ cargo run --bin sablier_merkle_api
```

To add another crate, you can run this command:

```sh
$ cd apps/merkle-api
$ cargo add crate_name
```

## API

Do not add trailing slashes to the API endpoints.

### Health

```
GET https://.../api/health
```

### Eligibility

```
GET https://.../api/eligibility?address=...&cid=...
```

### Create

```
POST https://.../api/create?decimals=... + FORM_DATA{file: "a csv file with addresses and amounts"}
```

### Csv Example

You can find a gist of a Rust CSV Generator at: https://gist.github.com/gavriliumircea/2a9797f207a2a2f3832ddaa376337e8c