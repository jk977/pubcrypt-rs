mod crypt;

use clap::{clap_app, AppSettings, ArgMatches};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{
    cmp,
    fmt::Display,
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
};

use crypt::{Block, Key, KeyPair, Num, BLOCK_BYTES, NUM_BYTES};

#[derive(Debug)]
struct CryptSettings {
    reader: BufReader<File>,
    writer: BufWriter<File>,
    key: Key,
}

/**
 * Get the inner `Ok` value of the result. If the result is `Err(_)`, print an informational message
 * prefixed with `msg` and exit the process.
 *
 * Returns the `Ok` value of `r`. Does not return if `r` is an error.
 */
fn ok_or_die<T, E: Display>(r: Result<T, E>, msg: &str) -> T {
    match r {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{}: {}", msg, e);
            std::process::exit(1);
        }
    }
}

/**
 * Generate a public-private key pair and write them to the values `PUB_OUT` and `PRIV_OUT` from
 * `matches`, respectively.
 */
fn gen_keys(matches: &ArgMatches) -> io::Result<()> {
    use math::primes::PrimeError;

    let pub_path = matches.value_of("PUB_OUT").unwrap();
    let priv_path = matches.value_of("PRIV_OUT").unwrap();
    let mut rng = StdRng::from_entropy();
    let keys = match KeyPair::generate(&mut rng) {
        Err(PrimeError::InvalidRange) => unimplemented!(),
        Err(PrimeError::PrimeNotFound) => unimplemented!(),
        Ok(k) => k,
    };

    let write_key = |path: &str, key: Key| fs::write(path, &key.serialize());
    write_key(pub_path, keys.public)?;
    write_key(priv_path, keys.private)?;
    Ok(())
}

/**
 * Check if the reader is at the end of the file by trying to fill its buffer, then
 * checking whether or not it's empty. If it is, the reader has reached EOF. Otherwise,
 * there are still bytes remaining.
 */
fn reader_is_eof<R: BufRead>(reader: &mut R) -> io::Result<bool> {
    reader.fill_buf().map(|buf| buf.is_empty())
}

/**
 * Encrypt the given block to the file specified in `settings`, using the
 * encryption key also present in `settings`.
 */
fn encrypt_block_to_file<T: Rng>(
    buf: [u8; BLOCK_BYTES],
    settings: &mut CryptSettings,
    rng: &mut T,
) -> io::Result<()> {
    let block = Block::from_be_bytes(buf);
    let (c1, c2) = crypt::encrypt_block(block, &settings.key, rng);

    settings.writer.write_all(&c1.to_be_bytes())?;
    settings.writer.write_all(&c2.to_be_bytes())?;
    Ok(())
}

/**
 * Fill unused space in the buffer with random bytes, and place the number of
 * pad bytes at the end of the buffer.
 *
 * Panics if `pad_bytes > buf.len()`.
 */
fn pad_block<T: Rng, const N: usize>(buf: &mut [u8; N], pad_bytes: u8, rng: &mut T) {
    assert!(pad_bytes as usize <= buf.len());

    let pad_start = buf.len() - pad_bytes as usize;
    let pad_end = buf.len() - 1;
    rng.fill(&mut buf[pad_start..pad_end]);
    buf[pad_end] = pad_bytes;
}

/**
 * Encrypt according to the configuration in `settings`. The plaintext is read
 * from `settings.reader`, encrypted with the public key `settings.key`, and
 * written to `settings.writer`.
 */
fn encrypt_ecb(mut settings: CryptSettings) -> io::Result<()> {
    let mut rng = StdRng::from_entropy();
    let mut buf = [0_u8; BLOCK_BYTES];
    let mut has_padded_block = false;

    while !reader_is_eof(&mut settings.reader)? {
        let reader_buf = settings.reader.fill_buf()?;
        let bytes_consumed = cmp::min(reader_buf.len(), buf.len());
        let pad_bytes = (buf.len() - bytes_consumed) as u8;

        // consume bytes directly from the reader's buffer since it doesn't
        // support reading a sequence of bytes shorter than or equal in length
        // to a given number
        buf[..bytes_consumed].copy_from_slice(&reader_buf[..bytes_consumed]);
        settings.reader.consume(bytes_consumed);

        // pad final byte with the length if not enough bytes were read
        if pad_bytes > 0 {
            pad_block(&mut buf, pad_bytes, &mut rng);
            has_padded_block = true;
        }

        encrypt_block_to_file(buf, &mut settings, &mut rng)?;
    }

    if !has_padded_block {
        // add an empty padded block at the end to prevent bad things when decrypting
        let pad_bytes = buf.len() as u8;
        pad_block(&mut buf, pad_bytes, &mut rng);
        encrypt_block_to_file(buf, &mut settings, &mut rng)?;
    }

    Ok(())
}

/**
 * Decrypt according to the configuration in `settings`. The ciphertext is read
 * from `settings.reader`, decrypted with the private key `settings.key`, and
 * written to `settings.writer`.
 */
fn decrypt_ecb(mut settings: CryptSettings) -> io::Result<()> {
    loop {
        let buf = settings.reader.fill_buf()?;

        if buf.len() < NUM_BYTES * 2 {
            let e_msg = format!(
                "Decrypted file must have a multiple of {} bytes",
                NUM_BYTES * 2
            );
            let e = io::Error::new(io::ErrorKind::UnexpectedEof, e_msg);
            return Err(e);
        }

        let mut c1_buf = [0_u8; NUM_BYTES];
        let mut c2_buf = [0_u8; NUM_BYTES];
        c1_buf.copy_from_slice(&buf[..NUM_BYTES]);
        c2_buf.copy_from_slice(&buf[NUM_BYTES..NUM_BYTES * 2]);
        settings.reader.consume(NUM_BYTES * 2);

        let c1 = Num::from_be_bytes(c1_buf);
        let c2 = Num::from_be_bytes(c2_buf);
        let decrypted = crypt::decrypt_block(c1, c2, &settings.key);
        let block_bytes = Block::to_be_bytes(decrypted);

        if reader_is_eof(&mut settings.reader)? {
            // last ciphertext block, so it contains a pad count
            let pad_bytes = block_bytes[block_bytes.len() - 1] as usize;
            assert!(pad_bytes <= block_bytes.len());

            let unpadded = &block_bytes[..block_bytes.len() - pad_bytes];
            return settings.writer.write_all(unpadded);
        } else {
            settings.writer.write_all(&block_bytes)?;
        };
    }
}

/**
 * Build the command-line application using `clap`.
 */
fn build_clap_app() -> clap::App<'static, 'static> {
    clap_app!(pubcrypt =>
        (author: "jk977")
        (about: "Public key encryption and decryption application")
        (setting: AppSettings::SubcommandRequiredElseHelp)
        (@subcommand genkey =>
            (@arg PUB_OUT: --pub +takes_value +required "Output public key to given file")
            (@arg PRIV_OUT: --priv +takes_value +required "Output private key to given file")
        )
        (@subcommand crypt =>
            (@group mode =>
                (@attributes +required)
                (@arg ENCRYPT: -e "Sets algorithm to encrypt")
                (@arg DECRYPT: -d "Sets algorithm to decrypt")
            )
            (@arg INPATH:
                -i --in +takes_value +required
                "Read the algorithm input from the given file"
            )
            (@arg OUTPATH:
                -o --out +takes_value +required
                "Write the algorithm output to the given file"
            )
            (@arg KEYPATH:
                -k --key +takes_value +required
                "Read the encryption/decryption key from the given file"
            )
        )
    )
}

fn get_crypt_settings(matches: &ArgMatches) -> CryptSettings {
    let in_file = {
        let in_path = matches.value_of("INPATH").unwrap();
        let err_msg = format!("Failed to open input file {}", in_path);
        ok_or_die(File::open(in_path), &err_msg)
    };
    let out_file = {
        let out_path = matches.value_of("OUTPATH").unwrap();
        let err_msg = format!("Failed to open output file {}", out_path);
        ok_or_die(File::create(out_path), &err_msg)
    };
    let key = {
        let key_path = matches.value_of("KEYPATH").unwrap();
        let mut key_file = ok_or_die(File::open(key_path), "Failed to open key file");
        let mut buf = [0u8; Key::KEY_BYTES];
        ok_or_die(
            key_file.read_exact(&mut buf),
            "Failed to read key from file",
        );
        Key::deserialize(&buf)
    };

    CryptSettings {
        reader: BufReader::new(in_file),
        writer: BufWriter::new(out_file),
        key,
    }
}

fn main() {
    let matches = build_clap_app().get_matches();

    if let Some(matches) = matches.subcommand_matches("genkey") {
        // `genkey` subcommand; generate public/private key pair
        ok_or_die(gen_keys(matches), "Failed to generate and write keys");
    } else if let Some(matches) = matches.subcommand_matches("crypt") {
        // `crypt` subcommand; handle encryption/decryption
        assert!(matches.is_present("ENCRYPT") || matches.is_present("DECRYPT"));
        let settings = get_crypt_settings(matches);

        if matches.is_present("ENCRYPT") {
            ok_or_die(encrypt_ecb(settings), "Encryption failed");
        } else {
            ok_or_die(decrypt_ecb(settings), "Decryption failed");
        }
    } else {
        unreachable!("Failed to cover all subcommands, or Clap is improperly configured");
    }
}
