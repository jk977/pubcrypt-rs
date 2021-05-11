Project 2: Public-Key Cryptography
==================================

About
-----

This project implements a public-key cryptography algorithm. A command-line interface is also provided for key generation, ECB encryption, and ECB decryption.

**NOTE**: This program was done for learning purposes and should not be relied on for cryptographic security. Use a well-established, cryptographically secure algorithm instead.

Usage
-----

The program usage is as follows:

    pubcrypt <-d|-e> [-in <INPATH>] [-out <OUTPATH>] [-k <KEYPATH>]
    pubcrypt -genkey

    Flags:
        -d               Encrypt INPATH and write result to OUTPATH
        -e               Encrypt INPATH and write result to OUTPATH
        -genkey          Generate public and private keys to pubkey.txt
                         and prikey.txt, respectively.
        -h, --help       Prints help information

    Options:
            -in <INPATH>      The file to read the algorithm input from.
                              Defaults to ptext.txt when encrypting and
                              ctext.txt when decrypting.
            -out <OUTPATH>    The file to write the algorithm output to.
                              Defaults to ctext.txt when encrypting and
                              ptext.txt when decrypting.
            -k <KEYPATH>      The file to read the key from. Defaults to
                              Defaults to pubkey.txt when encrypting and
                              prikey.txt when decrypting.

Examples
--------

Generate key pair for encryption and decryption:

    cargo run -- -genkey

Encrypt the file `foo.txt` with the generated public key and write the result to `foo.enc`:

    cargo run -- -e -k pubkey.txt -in foo.txt -out foo.enc

Decrypt the file `foo.enc` with the generated private key and write the result to `decrypted.txt`:

    cargo run -- -d -k prikey.txt -in foo.enc -out decrypted.txt
