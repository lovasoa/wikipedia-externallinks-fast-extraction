extern crate wikipedia_externallinks_fast_extraction;
extern crate rayon;

use std::io;
use wikipedia_externallinks_fast_extraction::iter_string_urls;
use rayon::iter::ParallelIterator;

fn main() {
    let stdin = io::stdin();
    iter_string_urls(stdin.lock()).for_each(|url_result| {
        match url_result {
            Ok(url) => println!("{}", url),
            Err(err) => eprintln!("{}", err),
        }
    });
}
