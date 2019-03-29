#[macro_use]
extern crate clap;
extern crate cartorio;

use cartorio::server;

use clap::{App, Arg};

fn main() {
    let matches = App::new("cartorio")
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .args(&[
            Arg::with_name("address")
                .default_value("0.0.0.0:5000")
                .short("a")
                .long("address")
                .help("IP address to listen for requests"),
            Arg::with_name("assets-dir")
                .default_value("/tmp/cartorio")
                .long("assets-dir")
                .help("Directory where registry blobs and configurations are saved to"),
            Arg::with_name("docker-tarball")
                .multiple(true)
                .long("docker-tarball")
                .help("Tarball from `docker-save` to unpack"),
        ])
        .get_matches();


    server::serve(&value_t!(matches, "address", String).unwrap());
}
