/// Handle command line arguments
mod cli;

/// Specify configuration file formats
mod config;

/// Map circuit data into bevy components
mod mapper;

/// Encode wires' channels using an integer
type Channel = u32;

fn main() {
    // For now, only command line interface is supported
    //let _ = cli::read_cli();

    // TODO: for now we are just going to pick the first model
    //let circuit = CircuitData::<usize>::generate::<VoxValue, 8>(&models[0], &config.wires, 3);

    println!("Hello, world!");
}
