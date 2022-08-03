// To run these tests
// cargo test benchmark --features benchmarking

#[test]
#[cfg_attr(not(feature= "benchmarking"), ignore)]
fn benchmark_1() {
    panic!("BAD TOUCH");
}