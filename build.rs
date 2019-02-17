extern crate glob;
extern crate protoc_rust;

use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use protoc_rust::Customize;

// Convert output of glob function to something compatible with protoc
// generation functions.
fn glob_simple(pattern: &str) -> Vec<String> {
    glob::glob(pattern)
        .expect("glob failed")
        .map(|g| {
            g.expect("item")
                .as_path()
                .to_str()
                .expect("utf-8")
                .to_owned()
        })
        .collect()
}

fn main() {
    let proto_src_files = glob_simple("protos/*.proto");
    println!("{:?}", proto_src_files);

    let out_dir = env::var("OUT_DIR").expect("No OUT_DIR env variable");
    let protos_path = Path::new(&out_dir).join("protos");
    fs::create_dir_all(&protos_path).expect("Unable to create proto destination directory");

    let mod_file_content = proto_src_files
        .iter()
        .map(|proto_file| {
            let proto_path = Path::new(proto_file);
            format!(
                "pub mod {};",
                proto_path
                    .file_stem()
                    .expect("Unable to extract proto path stem")
                    .to_str()
                    .expect("Unable to extract proto filename")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut mod_file = File::create(protos_path.join("mod.rs")).unwrap();
    mod_file
        .write_all(mod_file_content.as_bytes())
        .expect("Unable to write protos mod file.");

    protoc_rust::run(protoc_rust::Args {
        out_dir: &protos_path
            .to_str()
            .expect("Invalid proto destination path"),
        input: &proto_src_files
            .iter()
            .map(|a| a.as_ref())
            .collect::<Vec<&str>>(),
        includes: &["protos"],
        customize: Customize {
            ..Default::default()
        },
    })
    .expect("Unable to run protoc");
}
