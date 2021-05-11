mod crypt;
mod util;

use clap::{clap_app, AppSettings};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::{
    cmp,
    fs::{self, File},
    io::{self, BufRead, BufReader, BufWriter, Write},
};

use crypt::{Block, Key, KeyPair, BLOCK_BYTES};
use util::parse_words;

#[derive(Debug)]
struct CryptSettings {
    reader: BufReader<File>,
    writer: BufWriter<File>,
    key: Key,
}

/**
 * Generate a public-private key pair and write them to `pub_path` and
 * `priv_path`, respectively.
 */
fn gen_keys_to(pub_path: &str, priv_path: &str) -> io::Result<()> {
    let mut rng = StdRng::from_entropy();
    let KeyPair {
        public: pub_key,
        private: priv_key,
    } = KeyPair::generate(&mut rng);

    let write_key = |path: &str, key: Key| fs::write(path, key.to_string());
    write_key(pub_path, pub_key).and_then(|_| write_key(priv_path, priv_key))
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
    let encrypted_str = format!("{} {}\n", c1, c2);
    settings.writer.write_all(encrypted_str.as_bytes())
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
    let mut lines = settings.reader.lines().peekable();

    while let Some(line) = lines.next() {
        let [c1, c2] = parse_words(&line?)?;
        let decrypted = crypt::decrypt_block(c1, c2, &settings.key);

        let buf = Block::to_be_bytes(decrypted);
        let out_buf = if lines.peek().is_none() {
            // last ciphertext block, so it contains a pad count
            let pad_bytes = buf[buf.len() - 1] as usize;
            &buf[..buf.len() - pad_bytes]
        } else {
            &buf
        };

        settings.writer.write_all(out_buf)?;
    }

    Ok(())
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

fn main() -> io::Result<()> {
    let matches = build_clap_app().get_matches();

    if let Some(matches) = matches.subcommand_matches("genkey") {
        let pub_out = matches.value_of("PUB_OUT").unwrap();
        let priv_out = matches.value_of("PRIV_OUT").unwrap();
        gen_keys_to(pub_out, priv_out)
    } else if let Some(matches) = matches.subcommand_matches("crypt") {
        // handle encryption/decryption
        let crypt = if matches.is_present("ENCRYPT") {
            encrypt_ecb
        } else {
            assert!(matches.is_present("DECRYPT"));
            decrypt_ecb
        };

        let in_path = matches.value_of("INPATH").unwrap();
        let out_path = matches.value_of("OUTPATH").unwrap();
        let key_path = matches.value_of("KEYPATH").unwrap();
        let settings = CryptSettings {
            reader: BufReader::new(File::open(in_path)?),
            writer: BufWriter::new(File::create(out_path)?),
            key: fs::read_to_string(key_path)?.parse()?,
        };

        crypt(settings)
    } else {
        unreachable!("Failed to cover all subcommands, or Clap is improperly configured");
    }
}
