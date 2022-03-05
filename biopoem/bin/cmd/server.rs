use std::path::Path;
use structopt::StructOpt;

/// A collection of metadata, such as file size, md5sum
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="Biopoem - Server", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct Arguments {
  /// Bam file to process
  #[structopt(name = "FILE", multiple = true, takes_value = true)]
  inputs: Vec<String>,

  /// Output file.
  #[structopt(name = "output", short = "o", long = "output")]
  output: String,
}

pub fn run(args: &Arguments) {
  println!("{:?}", args);
}
