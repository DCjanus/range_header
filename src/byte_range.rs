//! reference: <https://tools.ietf.org/html/rfc7233>

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./byte_range.pest"]
struct ByteRangeParser;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct ByteRange {
    pub offset: u64,
    pub length: u64,
}

impl ByteRange {
    /// Parses Range HTTP header string as per RFC 2733,but `bytes` only
    /// With invalid input, return empty vector
    ///
    /// `header`: HTTP Range header (e.g. `bytes=0-100`)
    /// `full_size`: full size of the selected representation
    ///
    /// #Example
    ///
    /// ```Rust
    /// assert_eq!(
    ///     ByteRange::parse("bytes=10-100", 200),
    ///     ByteRange {
    ///         offset: 10,
    ///         length: 91,
    ///     }
    /// )
    /// ```
    pub fn parse(header: &str, full_size: u64) -> Vec<Self> {
        if full_size < 1 {
            return vec![];
        }

        let byte_range_spec_iter = match ByteRangeParser::parse(Rule::byte_ranges_specifier, header)
        {
            Err(_) => {
                return vec![];
            }
            Ok(x) => x.peek().unwrap().into_inner(),
        };

        let mut result = Vec::new();
        for spec in byte_range_spec_iter {
            match spec.as_rule() {
                Rule::from_to => {
                    // eg. '200-'
                    let offset: u64 = spec.into_inner().peek().unwrap().as_str().parse().unwrap();
                    if offset < full_size {
                        result.push(ByteRange {
                            offset,
                            length: full_size - offset,
                        })
                    }
                }
                Rule::from_to_all => {
                    // eg, '200-300'
                    let mut inner_pairs = spec.into_inner();
                    let begin: u64 = inner_pairs.next().unwrap().as_str().parse().unwrap();
                    let mut end: u64 = inner_pairs.next().unwrap().as_str().parse().unwrap();

                    if begin >= full_size {
                        continue;
                    }
                    if end >= full_size {
                        end = full_size - 1
                    }
                    if begin > end {
                        continue;
                    }
                    result.push(ByteRange {
                        offset: begin,
                        length: end - begin + 1,
                    })
                }
                Rule::last => {
                    // eg. '-200'
                    let length: u64 = spec.into_inner().peek().unwrap().as_str().parse().unwrap();
                    let length = length.min(full_size);
                    result.push(ByteRange {
                        offset: full_size - length,
                        length,
                    })
                }
                Rule::EOI => {}
                _ => unreachable!(),
            }
        }
        result
    }
}

#[test]
fn test_parse() {
    let test_cases: Vec<(&str, u64, Vec<ByteRange>)> = vec![
        ("", 0, vec![]),
        ("", 1000, vec![]),
        ("bytes=1-1", 0, vec![]),
        ("bytes=1-2", 1, vec![]),
        (
            "bytes=0-0",
            1,
            vec![ByteRange {
                offset: 0,
                length: 1,
            }],
        ),
        (
            "bytes=0-1",
            1,
            vec![ByteRange {
                offset: 0,
                length: 1,
            }],
        ),
        (
            "bytes=0-2",
            1,
            vec![ByteRange {
                offset: 0,
                length: 1,
            }],
        ),
        ("foo", 0, vec![]),
        ("bytes=", 0, vec![]),
        ("bytes=7", 10, vec![]),
        ("bytes= 7 ", 10, vec![]),
        ("bytes=1-", 0, vec![]),
        ("bytes=5-4", 10, vec![]),
        (
            "bytes=0-2,5-4",
            10,
            vec![ByteRange {
                offset: 0,
                length: 3,
            }],
        ),
        (
            "bytes=2-5,4-3",
            10,
            vec![ByteRange {
                offset: 2,
                length: 4,
            }],
        ),
        ("bytes=--5,4--3", 10, vec![]),
        ("bytes=A-", 10, vec![]),
        ("bytes=A- ", 10, vec![]),
        ("bytes=A-Z", 10, vec![]),
        ("bytes= -Z", 10, vec![]),
        ("bytes=5-Z", 10, vec![]),
        ("bytes=Ran-dom, garbage", 10, vec![]),
        ("bytes=0x01-0x02", 10, vec![]),
        ("bytes=         ", 10, vec![]),
        ("bytes= , , ,   ", 10, vec![]),
        (
            "bytes=0-9",
            10,
            vec![ByteRange {
                offset: 0,
                length: 10,
            }],
        ),
        (
            "bytes=0-",
            10,
            vec![ByteRange {
                offset: 0,
                length: 10,
            }],
        ),
        (
            "bytes=5-",
            10,
            vec![ByteRange {
                offset: 5,
                length: 5,
            }],
        ),
        (
            "bytes=0-20",
            10,
            vec![ByteRange {
                offset: 0,
                length: 10,
            }],
        ),
        (
            "bytes=15-,0-5",
            10,
            vec![ByteRange {
                offset: 0,
                length: 6,
            }],
        ),
        (
            "bytes=1-2,5-",
            10,
            vec![
                ByteRange {
                    offset: 1,
                    length: 2,
                },
                ByteRange {
                    offset: 5,
                    length: 5,
                },
            ],
        ),
        (
            "bytes=-2 , 7-",
            11,
            vec![
                ByteRange {
                    offset: 9,
                    length: 2,
                },
                ByteRange {
                    offset: 7,
                    length: 4,
                },
            ],
        ),
        (
            "bytes=0-0 ,2-2, 7-",
            11,
            vec![
                ByteRange {
                    offset: 0,
                    length: 1,
                },
                ByteRange {
                    offset: 2,
                    length: 1,
                },
                ByteRange {
                    offset: 7,
                    length: 4,
                },
            ],
        ),
        (
            "bytes=-5",
            10,
            vec![ByteRange {
                offset: 5,
                length: 5,
            }],
        ),
        (
            "bytes=-15",
            10,
            vec![ByteRange {
                offset: 0,
                length: 10,
            }],
        ),
        (
            "bytes=0-499",
            10000,
            vec![ByteRange {
                offset: 0,
                length: 500,
            }],
        ),
        (
            "bytes=500-999",
            10000,
            vec![ByteRange {
                offset: 500,
                length: 500,
            }],
        ),
        (
            "bytes=-500",
            10000,
            vec![ByteRange {
                offset: 9500,
                length: 500,
            }],
        ),
        (
            "bytes=9500-",
            10000,
            vec![ByteRange {
                offset: 9500,
                length: 500,
            }],
        ),
        (
            "bytes=0-0,-1",
            10000,
            vec![
                ByteRange {
                    offset: 0,
                    length: 1,
                },
                ByteRange {
                    offset: 9999,
                    length: 1,
                },
            ],
        ),
        (
            "bytes=500-600,601-999",
            10000,
            vec![
                ByteRange {
                    offset: 500,
                    length: 101,
                },
                ByteRange {
                    offset: 601,
                    length: 399,
                },
            ],
        ),
        (
            "bytes=500-700,601-999",
            10000,
            vec![
                ByteRange {
                    offset: 500,
                    length: 201,
                },
                ByteRange {
                    offset: 601,
                    length: 399,
                },
            ],
        ),
        //         Match Apache laxity:
        (
            "bytes=   1 -2   ,  4- 5, 7 - 8 , ,,",
            11,
            vec![
                ByteRange {
                    offset: 1,
                    length: 2,
                },
                ByteRange {
                    offset: 4,
                    length: 2,
                },
                ByteRange {
                    offset: 7,
                    length: 2,
                },
            ],
        ),
    ];
    for case in test_cases {
        let (header, full_size, expect) = case;
        let actual = ByteRange::parse(header, full_size);
        if actual != expect {
            panic!(format!(
                "header: {:?} full_size: {:?}\n expect: {:?}\n actual: {:?}",
                header, full_size, expect, actual
            ))
        }
    }
}
