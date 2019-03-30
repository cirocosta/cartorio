#[macro_use]
extern crate clap;
extern crate cartorio;

use cartorio::{server, loader};
use clap::{App, AppSettings, Arg, SubCommand};

fn main() {
    let matches = App::new("cartorio")
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("load")
                .about(
                    "Loads `docker save`d tarballs into cartorio's internal filesystem hierarchy",
                )
                .args(&[
                    Arg::with_name("blobstore")
                        .default_value("/tmp/cartorio/blobstore")
                        .short("b")
                        .long("blobstore")
                        .help("Directory where blobs, manifests and configurations are saved to"),
                    Arg::with_name("tarball")
                        .required(true)
                        .long("tarball")
                        .short("t")
                        .help("Tarball to load into the registry"),
                ]),
        )
        .subcommand(
            SubCommand::with_name("serve")
                .about("Serve loaded images as a Docker registry")
                .args(&[
                    Arg::with_name("address")
                        .default_value("0.0.0.0:5000")
                        .short("a")
                        .long("address")
                        .help("IP address to listen for requests"),
                    Arg::with_name("blobstore")
                        .default_value("/tmp/cartorio/blobstore")
                        .short("b")
                        .long("blobstore")
                        .help("Directory where blobs, manifests and configurations are saved to"),
                ]),
        )
        .get_matches();

    match matches.subcommand() {
        ("load", Some(m)) => {
            loader::load_tarball(
                &value_t!(m, "blobstore", String).unwrap(),
                &value_t!(m, "tarball", String).unwrap(),
            );
        }

        ("serve", Some(m)) => {
            server::serve(
                &value_t!(m, "address", String).unwrap(),
                &value_t!(m, "blobstore", String).unwrap(),
            );
        }

        ("", None) => eprintln!("No subcommand specified. Check --help."),

        _ => unreachable!(),
    }
}
