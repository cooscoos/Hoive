// To run these tests
// cargo test benchmark --features benchmarking -- --nocapture

#[test]
#[cfg_attr(not(feature= "benchmarking"), ignore)]
fn benchmark_1() {
    println!("BOOYAH");
    panic!("BAD TOUCH");
}