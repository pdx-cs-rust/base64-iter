pub fn encode<I>(input: I) -> impl Iterator<Item = char>
where
    I: IntoIterator<Item = u8>,
{
    const BASE_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    #[derive(Clone, Copy)]
    enum State {
        InProgress,
        Pad(usize),
        Done,
    }
    use State::*;

    fn decr_pad(pad: usize) -> State {
        if pad > 1 {
            Pad(pad - 1)
        } else {
            Done
        }
    }

    let mut input = input.into_iter().fuse();
    let mut bits = 0u16;
    let mut nbits = 0;
    let mut rem = 0;
    let mut state = InProgress;

    std::iter::from_fn(move || {
        if nbits < 6 {
            if let Some(b) = input.next() {
                bits = (bits << 8) | b as u16;
                nbits += 8;
                rem = (rem + 1) % 3;
            } else {
                match state {
                    InProgress => {
                        if nbits > 0 {
                            bits <<= 6 - nbits;
                            nbits = 6;
                        } else {
                            let pad = (3 - rem) % 3;
                            if pad > 0 {
                                state = decr_pad(pad);
                                return Some('=');
                            }
                            state = Done;
                            return None;
                        }
                    }
                    Pad(pad) => {
                        state = decr_pad(pad);
                        return Some('=');
                    }
                    Done => {
                        return None;
                    }
                }
            }
        }

        let idx = (bits >> (nbits - 6)) & 0x3f;
        nbits -= 6;
        Some(BASE_CHARS[idx as usize] as char)
    })
}

#[test]
fn test_encode() {
    let cases = [
        ("", ""),
        ("\0", "AA=="),
        ("\0\0", "AAA="),
        ("\0\0\0", "AAAA"),
        ("Rus", "UnVz"),
        ("Rust", "UnVzdA=="),
        ("Rus\0", "UnVzAA=="),
        ("Rust\0", "UnVzdAA="),
        ("Rusty", "UnVzdHk="),
        ("Rusty!", "UnVzdHkh"),
    ];

    for (plain, cipher) in cases {
        let encoded: String = encode(plain.bytes()).collect();
        assert!(
            cipher == encoded,
            "{:?}: expected {:?} got {:?}",
            plain,
            cipher,
            encoded,
        );
    }
}
