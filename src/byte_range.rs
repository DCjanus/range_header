//! reference: <https://tools.ietf.org/html/rfc7233>

use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./byte_range.pest"]
struct ByteRangeParser;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum ByteRange {
    FromTo(u64),
    FromToAll(u64, u64),
    Last(u64),
}

impl ByteRange {
    /// Parses Range HTTP header string as per RFC 2733,but `bytes` only.
    /// With invalid input, return empty vector
    ///
    /// # Examples
    ///
    /// ```rust
    /// use range_header::ByteRange;
    /// assert_eq!(
    ///     ByteRange::parse("bytes=10-100"),
    ///     vec![ByteRange::FromToAll(10, 100)]
    /// );
    ///
    /// assert_eq!(
    ///     ByteRange::parse("bytes=10-"),
    ///     vec![ByteRange::FromTo(10)]
    /// );
    ///
    ///assert_eq!(
    ///     ByteRange::parse("bytes=-100"),
    ///     vec![ByteRange::Last(100)]
    /// );
    ///
    /// assert_eq!(
    ///     ByteRange::parse("invalid input"),
    ///     vec![]
    /// );
    /// ```
    pub fn parse(header: &str) -> Vec<Self> {
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
                    result.push(ByteRange::FromTo(offset));
                }
                Rule::from_to_all => {
                    // eg, '200-300'
                    let mut inner_pairs = spec.into_inner();
                    let begin: u64 = inner_pairs.next().unwrap().as_str().parse().unwrap();
                    let end: u64 = inner_pairs.next().unwrap().as_str().parse().unwrap();

                    if begin > end {
                        continue;
                    }
                    result.push(ByteRange::FromToAll(begin, end));
                }
                Rule::last => {
                    // eg. '-200'
                    let length: u64 = spec.into_inner().peek().unwrap().as_str().parse().unwrap();
                    result.push(ByteRange::Last(length));
                }
                Rule::EOI => {}
                _ => unreachable!(),
            }
        }
        result
    }
}
