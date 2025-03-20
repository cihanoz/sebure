mod dpos;

fn main() {
    println!("Running SEBURE Blockchain tests...");
    
    // Run DPoS consensus tests
    dpos::run_tests();
    
    println!("All tests completed successfully!");
}
