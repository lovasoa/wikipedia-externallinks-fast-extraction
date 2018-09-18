extern crate wikipedia_externallinks_fast_extraction;

use std::io;
use wikipedia_externallinks_fast_extraction::iter_string_urls;

fn main() {
    let stdin = io::stdin();
    for url_result in iter_string_urls(stdin.lock()) {
        match url_result {
            Ok(url) => println!("{}", url),
            Err(err) => eprintln!("{}", err),
        }
    }
}
