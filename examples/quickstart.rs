// Run from the repository root: cargo run -p semanticant-examples --example quickstart
use semanticant_pack::Pack;
use semanticant_value::{NumberRules, parse_numbers};
fn main() {
    let pack = Pack::open(std::path::Path::new("packs/en/number-words")).unwrap();
    let rules = NumberRules::from_pack(&pack, '.', &[',']).unwrap();
    for n in parse_numbers("one hundred and twenty-three apples", &rules) {
        println!("{}", n.value.value);
    }
}
