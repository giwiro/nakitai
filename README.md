```
             _    _ _        _ 
 _ __   __ _| | _(_) |_ __ _(_)
| '_ \ / _` | |/ / | __/ _` | |
| | | | (_| |   <| | || (_| | |
|_| |_|\__,_|_|\_\_|\__\__,_|_|                                                         
```
[![made-with-Rust](https://img.shields.io/badge/Made%20with-Rust-orange.svg)](http://commonmark.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](https://opensource.org/licenses/MIT)


## Description

**nakitai** is a rust multithread ransomware that encrypts each file with chacha20poly1305 which, besides
from stream (online) encrypting the files by chunks, it's also nonce-reuse
misuse-resistant (as stated in this [paper](https://eprint.iacr.org/2015/189.pdf)) and verify the chunk integrity (
authentication) with poly1350.

## What's inside ?

This project compiles 2 binaries:

1. `ransomware`: Program that will perform the encryption of the files.

2. `rescue`: GUI Program that will decrypt all the encrypted files.

![recover](assets/recover.png?raw=true)

## Compile



## Process

## Legal Disclaimer

```
The author does not hold any responsibility for the bad use of this tool,
remember that attacking targets without prior consent is illegal and punished by law.
```