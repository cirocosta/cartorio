#[macro_use] extern crate clap;

use cartorio::blobstore::BlobStore;
use cartorio::docker_saved_tarball::DockerSavedTarball;
use cartorio::server;
use clap::{App, AppSettings, Arg, SubCommand};
use std::path::Path;

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
                    Arg::with_name("docker-save-tarball")
                        .value_name("TARBALL")
                        .takes_value(true)
                        .long("docker-save-tarball")
                        .help("Tarball to load into the registry"),
                    Arg::with_name("oci-image-layout")
                        .value_name("DIRECTORY")
                        .takes_value(true)
                        .long("oci-image-layout")
                        .help("Directory where an OCI Image Layout exists"),
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
            let mut something_loaded = false;

            let blobstore = BlobStore::new(
                Path::new(&value_t!(m, "blobstore", String).unwrap()),
            ).unwrap();

            if let Ok(docker_saved_tarball) = &value_t!(m, "docker-save-tarball", String) {
                let loader = DockerSavedTarball::new(
                    Path::new(docker_saved_tarball), blobstore,
                ).unwrap();

                if let Err(err) = loader.load() {
                    panic!("failed to load docker tarball - {}", err);
                }

                something_loaded = true;
            }

            if let Ok(oci_image_layout) = &value_t!(m, "oci-image-layout", String) {
                unimplemented!("TBD");

                something_loaded = true;
            }

            if !something_loaded {
                println!("error: must specify something to be loaded");
                std::process::exit(1);
            }
        }


        ("serve", Some(m)) => {
            let blobstore = BlobStore::new(
                Path::new(&value_t!(m, "blobstore", String).unwrap()),
            ).unwrap();

            server::serve(
                &value_t!(m, "address", String).unwrap(),
                blobstore,
            );
        }


        _ => unreachable!(),
    }
}
