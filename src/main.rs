use std::env;
use whereis;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Bitte Suchbegriff eingeben");
    } else {
        let query = &args[1];
        whereis::search(query);
    }
}
