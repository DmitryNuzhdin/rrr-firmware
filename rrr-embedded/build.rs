use std::fs::{File, read_dir};
use std::io::{BufReader, copy};
use std::path::PathBuf;
use trunk_build_time::cmd::build;
use trunk_build_time::config;
use embuild::{
    build::LinkArgs,
};
use flate2::Compression;
use tokio;
use flate2::write::GzEncoder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut cfg = config::ConfigOptsBuild::default();
    cfg.release = true;
    cfg.target = Some(PathBuf::from("../rrr-frontend/index.html"));
    cfg.filehash = Some(false);
    println!("{:?}", cfg);
    build::Build{build: cfg}.run(None).await.unwrap();

    let files = read_dir("../rrr-frontend/dist").unwrap();

    for file in files {
        let mut filename = file.unwrap().path().as_os_str().to_owned();

        let mut input = BufReader::new(File::open(&filename).unwrap());
        filename.push(".gz");
        let output = File::create(filename).unwrap();
        let mut encoder = GzEncoder::new(output, Compression::default());
        copy(&mut input, &mut encoder).unwrap();
        encoder.finish().unwrap();
    }

    LinkArgs::output_propagated("ESP_IDF")?;
    Ok(())
}