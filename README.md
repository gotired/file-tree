# File-Tree

This repository is build for convert git files change path into file tree

## Setup

need [`rust`](https://rust-lang.org/tools/install/) for compile the code

```sh
cargo build --release
```

if we want to use global after compile copy to our local bin (mac)

```sh
sudo cp -rv ./target/release/file-tree /usr/local/bin/file-tree
```

and validate by

```sh
which file-tree
```

Output should be

```sh
/usr/local/bin/file-tree
```

## To use

download binary file from releases and copy to local bin (mac)

```sh
sudo cp -rv file-tree-arm64 /usr/local/bin/file-tree &&
sudo chmod +x file-tree
```

### CLI Mode

we can create file change to be `.txt` file

```sh
cat example.txt | file-tree
```

the output will be in the terminal like

```sh
├── folder/
|   └── file_1.rs
└── file_2.rs
```

if we want to use files change from git 

```sh
git diff --name-only main | file-tree
```

if we want output as a file 

```sh
cat example.txt | file-tree > example-output.txt
```

if we want output to be in out clipboard (mac)

```sh
cat example.txt | file-tree | pbcopy
```

### TUI Mode

```sh
file-tree --tui
```
