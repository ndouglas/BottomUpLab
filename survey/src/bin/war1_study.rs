#[path = "../war1_study/mod.rs"]
mod war1_study;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if let Err(failure) = war1_study::cli::main_args(&args) {
        eprintln!("{}", failure.message);
        std::process::exit(failure.exit_code);
    }
}
