#[test]
fn tdd_ratchet_gatekeeper() {
    if std::env::var("TDD_RATCHET").is_err() {
        panic!("This project uses strict TDD via cargo ratchet. Do not run cargo test directly.");
    }
}
