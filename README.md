# Roc-Micro:bit

A roc platform for running on the microbit embedded processor.
A lot of stuff is still at the begining and being figured out.
Hopefully it will offer an Arduino like experience via Roc.

### To Use

- Setup git submodules (needed for roc)
```
git submodule init
git submodule update
```

- Install `probe-run` with defmt support.

```
cargo install probe-run
```

- Build roc (requires [nix](https://nixos.org/download.html)).

```
./build-roc.sh
```

- Deploy an app.

```
DEFMT_LOG=info ./deploy-app.sh prime
```


### Pre-commit hooks

This project is using pre-commit. Please run `pre-commit install` to install the git pre-commit hooks on your clone. Instructions on how to install pre-commit can be found [here](https://pre-commit.com/#install).

Every time you try to commit, pre-commit will run checks on your files to make sure they follow style standards and they aren't affected by some simple issues. If the checks fail, pre-commit won't let you commit.
