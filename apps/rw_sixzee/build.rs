use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Read;

fn main() {
    // Re-run this build script whenever the quotes JSON changes.
    println!("cargo:rerun-if-changed=assets/grandma_quotes.json");

    let mut file =
        std::fs::File::open("assets/grandma_quotes.json").expect("grandma_quotes.json not found");
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).expect("read grandma_quotes.json");

    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    let hash = hasher.finish();

    // Exposed as env!("GRANDMA_QUOTES_HASH") in the crate.
    println!("cargo:rustc-env=GRANDMA_QUOTES_HASH={hash:016x}");
}
