use std::{io, mem::MaybeUninit, str::FromStr};

/**
 * Parse `N` values of type `T` from `s`, delimited by whitespace, and return the
 * resulting array of values.
 */
pub fn parse_words<T: FromStr, const N: usize>(s: &str) -> io::Result<[T; N]> {
    macro_rules! make_data_err {
        ($msg: literal) => {
            io::Error::new(io::ErrorKind::InvalidData, $msg)
        };
    }

    let mut vals = MaybeUninit::<[T; N]>::uninit();
    let mut word_count = 0;

    for word in s.split_whitespace().map(str::parse) {
        if word_count == N {
            return Err(make_data_err!("too many values encountered while parsing"));
        }

        let val = word.map_err(|_| make_data_err!("failed to parse data"))?;

        unsafe {
            (vals.as_mut_ptr() as *mut T).add(word_count).write(val);
        }

        word_count += 1;
    }

    if word_count < N {
        Err(make_data_err!("not enough values to parse"))
    } else {
        unsafe { Ok(vals.assume_init()) }
    }
}
