mod cli;

fn main() {
    // For now, only command line interface is supported
    let _ = cli::read_cli();

    println!("Hello, world!");
}
