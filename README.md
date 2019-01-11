# Ranger Header

[![dependency status](https://deps.rs/repo/github/dcjanus/range_header/status.svg)](https://deps.rs/repo/github/dcjanus/range_header)

HTTP `Range` header parser, powered by [pest](https://github.com/pest-parser/pest).

# Example

```rust
use range_header::byte_range::ByteRange;

fn main(){
    assert_eq!(
        ByteRange::parse("bytes=10-100", 200),
        ByteRange {
            offset: 10,
            length: 91,
        }
    )
}
```