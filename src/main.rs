#[macro_use]
extern crate clap;
extern crate cartorio;

use cartorio::{server};
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
                        .default_value("./image.tar")
                        .long("tarball")
                        .short("t")
                        .help("Tarball to load into the registry"),
                ]),
        )
        .subcommand(
            SubCommand::with_name("pull")
                .about(
                    "Loads `docker save`d tarballs into cartorio's internal filesystem hierarchy",
                )
                .args(&[
                    Arg::with_name("url")
                        .default_value("https://registry-1.docker.io")
                        .short("u")
                        .long("url")
                        .help("URL of the registry"),
                    Arg::with_name("name")
                        .required(true)
                        .takes_value(true)
                        .short("n")
                        .long("name")
                        .help("Name of the image to pull"),
                    Arg::with_name("reference")
                        .default_value("latest")
                        .long("r")
                        .short("reference")
                        .help("Image reference - tag or digest (including `sha256:`)"),
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
            unimplemented!("not ready");
        }

        ("pull", Some(_m)) => {
            unimplemented!("not ready!");
        }

        ("serve", Some(m)) => {
            server::serve(
                &value_t!(m, "address", String).unwrap(),
                &value_t!(m, "blobstore", String).unwrap(),
            );
        }

        _ => unreachable!(),
    }
}
