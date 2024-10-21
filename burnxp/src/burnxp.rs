use burnxp_core::run;

fn main() {
    run(std::env::args().next().as_deref().unwrap_or("burnxp"));
}
