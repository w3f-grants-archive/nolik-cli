# Nolik CLI

## SETUP

Download the CLI app

`git clone https://github.com/chainify/nolik-cli`

### Config file
Config file stores user's wallets and accounts.
It is automatically generated by the app and located at the address:

`~/.nolik/config.toml`

Wallet config:
* `alias` - a short one-word name for a wallet
* `public` - a public key of a wallet in AccountId32 format 
* `seed` - a seed phrase of a wallet. Keep it safe and do not share it with others!
* `bs68seed` - wallet seed phrase in Base58 format. This is used if you need to import the wallet

Account config:
* `alias` - a short one-word name for a wallet
* `public` - a public key of an account in Base 58 format. This is the *address* of an account and can be shared with others
* `seed` - a seed phrase of an account. Keep it safe and do not share it with others!
* `bs68seed` - account seed phrase in Base58 format. This is used if you need to import the account
 

### Index file
Index file stores your outgoing and incoming messages.
It is automatically generated by the app and located at the address:

`~/.nolik/index.toml`

Normally messages should be stored in a local database, but this app is a proof of concept.

Message data:
* `index` - the index of a message
* `public` - the public key of your account (sender or recipient)
* `hash` - an IPFS ID of the message
* `nonce` - the unique secret nonce of a message
* `from` - the sender of a message
* `to` - message recipients

## Using the app
To use the app you will have to generate at least one Wallet and one Account.

### Generate a new Wallet
The wallet is used to hold the tokens and pay for sending the messages

`cargo run -- add wallet --alias personal`

Valid flags:
* `--alias` - is a short one-word name for your wallet (Required, Unique)
* `--import` - import the Base58 encrypted seed phrase for your wallet (Required, Unique)

### Generate a new account
Accounts are used for sending and receiving encrypted messages.
Each account has a pair of a public and a private key.
The public key is an address of an account.
You can create as many accounts as you want. 

`cargo run -- add account --alias alice`

Valid flags:
* `--alias` - is a short one-word name for your account (Required, Unique) 
* `--import` - import the Base58 encrypted seed phrase for your wallet (Required, Unique)

### Get coins
For demo purposes you can get free coins from Alice account.

`cargo run -- get coins -w personal`

Valid flags:
* `-w | --wallet` - an alias or a Base58 public key of your wallet (Required, Unique)

### Adding an owner to an account
Each account has to have an owner (the wallet). 
That is required to make sure that the wallet has a right to broadcast the message on behalf of a particular account.
One account can have multiple owners.
You can find more info about account owners in a [Nolik pallet description](https://github.com/chainify/pallet-nolik#addowner).

`cargo run -- add owner -a alice -w personal`

Valid flags:
* `-a | --account` - the account alias or a Base58 public key that needs new owner (Required, Unique)
* `-w | --wallet` - the wallet alias or a Base58 public that will be added as an owner (Required, Unique)

### Composing the message
Before sending the message should be composed first. 
At this stage the message is saved to the IPFS network as a batch file.
It contains the same message encrypted for each recipient.

`cargo run -- compose message -s alice -r Gq5xd5c62w4fryJx8poYexoBJAy9JUpjir9vR4qMDF8z -k SUBJECT -v testing -k BODY -v hello_world`

Valid flags:
* `-s | --sender` - the sender of a message. The alias or a Base58 public key for your account (Require, Unique)
* `-r | --recipient` - the recipient of a message. The Base58 public key of the recipient (Required, Non-Unique) 
* `-k | --key` - the key of the message. This attribute requires a corresponding Value (Optional, Non-Unique)
* `-v | --value` - the value of a message. This attribute requires a corresponding Key (Optional, Non-Unique)

In case of successful message composing you will get an IPFS hash of the saved file (for instance, `QmTFiymizv6yTBLbHTkWe7h4Giy9yg623rjMoMeQk29hf3`).
That hash is required for sending the message.
No need to copy it manually because it has already been copied to clipboard.
Just use `commad-V` to paste it.

### Send the message
The message can be sent only once, because each message has a unique hash which is saved to blockchain.
That hash is checked each time before validating the message delivery.
In order to successfully send the message the sender should not be in the Blacklist of a recipient.
In case if the recipient has a Whitelist, the sender's address should be included in it.

`cargo run -- send message -w personal -h QmTFiymizv6yTBLbHTkWe7h4Giy9yg623rjMoMeQk29hf3`

Valid keys:
* `-w | --wallet` - the wallet alias or a Base58 public key of an owner of a sender's address (Required, Unique)
* `-h | --hash` - the IPFS hash of a saved composed file (Required, Unique)

### Update the Blacklist
A Blacklist is a set of addresses that do not have a right to send the message to the recipient's address.
By default, the Blacklist of an account does not contain any addresses.
For now there is only an option to add new senders' addresses.
The address cannot be added to a Blacklist or a Whitelist at the same time.

`cargo run -- update blacklist --add QmTFiymizv6yTBLbHTkWe7h4Giy9yg623rjMoMeQk29hf3 --for alice -w personal`

Valid keys:
* `--for` - the alias or a Base58 public key of an account for which the Blacklist will be updated (Required, Unique)
* `--add` - the Base58 public key of an account address that will be added to the Blacklist (Required, Unique)
* `-w | --wallet` - an alias or a Base58 public key of the owner's wallet

### Update the Whitelist
A Whitelist is a set of addresses that have a right to send the message to the recipient's address.
By default, the Whitelist of an account does not contain any addresses.
For now there is only an option to add new senders' addresses.
The address cannot be added to a Blacklist or a Whitelist at the same time.

Valid keys:
* `--for` - the alias or a Base58 public key of an account for which the Whitelist will be updated (Required, Unique)
* `--add` - the Base58 public key of an account address that will be added to the Whitelist (Required, Unique)
* `-w | --wallet` - an alias or a Base58 public key of the owner's wallet
 
### Get the messages
Getting messages for you provided account.
Please notice that you will get both outgoing and incoming messages.

Valid keys:
* `-a | --account` - the alias or a Base58 public key of your account (Required, Unique)

## Testing

For testing you have to run a local blockchain and an IPFS node.

1. Go to your [Nolik](https://github.com/chainify/nolik) directory
2. Stop running nodes if any 

`docker compose stop`

3. Launch the node in dev mode

`docker compose -f docker-compose.dev.yml up -d`

4. Use Rust's native cargo command to run tests
 
`cargo test -- --test-threads=1`