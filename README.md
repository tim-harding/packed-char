[![docs.rs](https://img.shields.io/docsrs/packed-char)](https://docs.rs/packed-char/latest/)
[![Crates.io Version](https://img.shields.io/crates/v/packed-char)](https://crates.io/crates/packed-char/)
[![GitHub License](https://img.shields.io/github/license/tim-harding/packed-char)](https://choosealicense.com/licenses/mit/)

# packed-char

Allows either a `char` or a 22-bit integer to be stored in 32 bits, the same
size as a `char`.

## How it works

`packed-char` takes advantage of the valid ranges for a `char` to determine what
type of data is stored. These ranges are `0..0xD800` and `0xDFFF..0x10FFFF` (see
the documentation for
[`char`](https://doc.rust-lang.org/std/primitive.char.html)). The range
`0xD800..=0xDFFF` contains surrogate code points, which are not valid UTF-8
characters. `char`s are stored unmodified. To store a `u22` without overlapping
valid `char` ranges, it is first split it into two 11-bit chunks. The left chunk
is stored in the leading bits, which `char`s never overlap with. The right chunk
is stored in the trailing bits, which do overlap the bits used by `char`s. To
make this work, take note of the bit pattern in the surrogate range:

```text
1101100000000000 // Start
1101111111111111 // End
^^^^^
```

The leading 5 bits are constant in this range. Referred to here as the surrogate
mask, they serve as a signature for `u22` values. They are set along with the
left and right 11-bit chunks:

```text
11111111111  00000    11011            11111111111
left chunk | unused | surrogate mask | right chunk
```

Now we have two cases:

- The left chunk is zero and the value is in the surrogate range. 
- The left chunk is nonzero and the value exceeds `char::MAX`.

Thus, `char` and `u22` values are disambiguated.
