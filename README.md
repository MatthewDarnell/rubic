

# Rubic
A Qubic Wallet Written in Rust

### What is This?
This is a wallet software intended for storing seeds and addresses for the Qubic Cryptocurrency. It allows for encrypting your addresses as well as initiating transactions.

### What machines can build this?
Rubic is currently working on Windows desktops as well as Mac M1 series laptops.

### How to build?

First, install rust.

`Windows: https://rust-lang.org/tools/install`

`Linux/Mac: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`


Then build with the available scripts:

```agsl
Windows:
> build-win.bat
> run-win.bat
```
```agsl
Mac:
$ ./run-mac-m1.sh
```

Rubic will run a server at `localhost:3000`. You have the option to change this (as well as override several other options) in `.env`
### How to use?

Open `ui/index.html` in your browser. Incognito mode is recommended to avoid possible malicious extensions accessing your seeds.

In the browser, you have several options:

1. Create Random / Import Qubic Identities (Addresses)
2. Add Trusted Network Peers (Several Defaults Are Hardcoded)
3. Use Settings To Set a Master Password as well as Encrypt Wallet and Export To a .csv File


### Where is this Storing My Qubic Identities?

Rubic Creates a folder, by default `.rubic/` in your home directory. 

On Windows this is `C:/Users/<username>/.rubic/`, While on Mac/Linux it is `~/.rubic`

This Directory is able to be changed in `.env`


### How is this Storing My Qubic Identities?
In the created directory it creates a log file as well as a SQLite database file used for storing Identities, Seeds, Transactions, Tick data and more.

`Seeds` (when they are encrypted by storing a Master Password and Encrypting in Settings or at Import time)
are encrypted using [bcrypt](https://en.wikipedia.org/wiki/Bcrypt) 


### How can I get involved?
Join The Qubic Discord or Make a Pull Request

### Disclaimer
This software comes with no warranty, real or implied. Secure storage of seeds and passwords is paramount; total loss of funds may ensue otherwise.

This is a personal project, no guarantees on performance are made!