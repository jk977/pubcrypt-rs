Public-Key Cryptography
=======================

About
-----

This project implements a public-key cryptography algorithm. A command-line interface is also provided for key generation, ECB encryption, and ECB decryption.

**NOTE**: This program was done for learning purposes; it is most likely not cryptographically secure. Use a well-established, secure algorithm instead.

Usage
-----

The program usage is as follows:

    KEY GENERATION:

        pubcrypt genkey --priv <PRIV_OUTPATH> --pub <PUB_OUTPATH>

        Options:
            --priv <PRIV_OUTPATH>    Writes private key to the given path
            --pub <PUB_OUTPATH>      Writes public key to the given path

    ENCRYPTION AND DECRYPTION:

        pubcrypt crypt (-d|-e) --in <INPATH> --out <OUTPATH> --key <KEYPATH>

        Flags:
            -d               Decrypts INPATH and writes the result to OUTPATH
            -e               Encrypts INPATH and writes the result to OUTPATH

        Options:
            --in <INPATH>     Sets the file to read the algorithm input from
            --out <OUTPATH>   Sets the file to write the algorithm output to
            --key <KEYPATH>   Sets the file to read the key from

Examples
--------

Generate key pair for encryption and decryption:

    pubcrypt genkey --priv privkey.txt --pub pubkey.txt

Encrypt the file `foo.txt` with the generated public key and write the result to `foo.enc`:

    pubcrypt crypt -e --key pubkey.txt --in foo.txt --out foo.enc

Decrypt the file `foo.enc` with the generated private key and write the result to `decrypted.txt`:

    pubcrypt crypt -d --key privkey.txt --in foo.enc --out decrypted.txt
