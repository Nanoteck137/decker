# Decker

CLI Tools for for the Value Steam Deck written in Rust

## Description

Decker is a collection of tools to interact and develope for the Steam Deck.

Based on the official SteamOS devkit client from Value but made to work on Windows/Linux/MacOS.

## Getting Started

### Dependencies

* [Rust compiler](https://www.rust-lang.org/tools/install)

Add musl rust target
```bash
rustup target add x86_64-unknown-linux-musl
```

#### MacOS
Install musl cross compiler
```bash
brew install FiloSottile/musl-cross/musl-cross
```

### Build

```bash
make
```

### Build and Run
```bash
make run
make run ARGS="--help" # run with arguments
```

### Install
```bash
make install
```

### Usage
Deploy an app to the Steam Deck
```bash
decker -d <Steam Deck IP> deploy
decker -d <Steam Deck IP> deploy --help # Print the full help text

# Example of a deployment
decker -d <Steam Deck IP> deploy "Test Game" run_game.sh ./game_files
```

Start a SSH session with the Steam Deck
```bash
decker -d <Steam Deck IP> shell
```


## Authors

Patrik Millvik Rosenström <patrik.millvik@gmail.com>

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

## Acknowledgments

* [Steam Deck Homepage](https://www.steamdeck.com)
* [Value Official SteamOS devkit tools](https://gitlab.steamos.cloud/devkit/steamos-devkit)
