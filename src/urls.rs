/*
 * Copyright (c) 2016-2018 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

use std::env;
use std::path::Path;

use url::{Position, Url};
use url::percent_encoding::percent_decode;

/// Get a file URL from the input if the file exists, otherwise return the input as is.
pub fn canonicalize_url(url: &str) -> String {
    if Path::new(url).exists() {
        if let Ok(path) = env::current_dir() {
            let url = path.join(url);
            if let Some(url) = url.to_str() {
                return format!("file://{}", url);
            }
        }
    }
    url.to_string()
}

/// Get the base URL (domain and tld) of an URL.
pub fn get_base_url(url: &str) -> Option<String> {
    Url::parse(url).ok().map(|parsed_url| {
        let mut base_url = parsed_url.host_str().unwrap_or("").to_string();
        // Remove all sub-domains.
        let mut period_count = base_url.chars().filter(|&c| c == '.').count();
        while period_count > 1 {
            base_url = base_url.chars().skip_while(|&c| c != '.').skip(1).collect();
            period_count = base_url.chars().filter(|&c| c == '.').count();
        }
        base_url
    })
}

/// Get the filename from the URL.
pub fn get_filename(url: &str) -> Option<String> {
    Url::parse(url).ok()
        .and_then(|parsed_url|
              parsed_url.path_segments()
                  .and_then(|segments| segments.last())
                  .and_then(|filename| percent_decode(filename.as_bytes()).decode_utf8().ok())
                  .map(|string| string.into_owned())
        )
}

pub fn host(url: &str) -> Option<String> {
    Url::parse(url).ok().and_then(|parsed_url| {
        parsed_url.host_str().map(|host| host.to_string())
    })
}

/// Check if the input string looks like a URL.
pub fn is_url(input: &str) -> bool {
    Url::parse(input).is_ok() || (Url::parse(&format!("http://{}", input)).is_ok() &&
                                  (input.contains('.') || input.contains(':'))) ||
        input == "localhost"
}

/// Take url and increment the first number with offset.
pub fn offset(url: &str, inc_offset: i32) -> Option<String> {
    let url =
        if url.ends_with('?') {
            url.trim_end_matches('?')
        }
        else {
            url
        };
    if let Ok(url) = Url::parse(&url) {
        let mut updated = false;

        if let Some(query) = url.query() {
            let pairs = url.query_pairs().into_owned();

            // ParseIntoOwned lacks DoubleEndedIterator for rev(), must be parsed left to right
            let next = pairs.map(|(lhs, rhs)| {
                if updated {
                    lhs + "=" + &rhs
                }
                else if let Ok(number) = rhs.parse::<i32>() {
                    updated = true;
                    lhs + "=" + (number + inc_offset).to_string().as_str()
                }
                else {
                    lhs + "=" + &rhs
                }
            }).fold(String::new(), |acc, ref x| {
                if acc.is_empty() {
                    acc + &x
                }
                else {
                    acc + "&" + &x
                }
            });

            if updated {
                return Some(url[..Position::BeforeQuery].to_string() + &next);
            }
            else {
                if let Some(page) = offset(&url[..Position::BeforeQuery], inc_offset) {
                    return Some(page + "?" + query);
                }
            }
        }
        else if let Some(path_segments) = url.path_segments() {
            let next = path_segments
                .rev() // check in reverse
                .map(|segment| {
                    if !updated {
                        if let Ok(number) = segment.parse::<i32>() {
                            updated = true;
                            return String::from("/") + (number + inc_offset).to_string().as_str();
                        }
                    }

                    String::from("/") + segment
                })
                .rev() // reverse again to normal state
                .collect::<String>();

            if updated {
                return Some(url[..Position::BeforePath].to_string() + &next);
            }
            else {
                // TODO: Check for some edge cases with a regex or tokenizer, ie: example.com/page6.
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::offset;

    #[test]
    fn test_offset() {
        assert_eq!(offset("https://www.mexicoinmykitchen.com/page/2/", 1), Some("https://www.mexicoinmykitchen.com/page/3/".to_string()));
        assert_eq!(offset("https://www.mexicoinmykitchen.com/page/2/?s=spicy", 1), Some("https://www.mexicoinmykitchen.com/page/3/?s=spicy".to_string()));
    }
}
