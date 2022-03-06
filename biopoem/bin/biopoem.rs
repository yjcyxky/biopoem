#[macro_use]
extern crate log;
extern crate lazy_static;

mod cmd;

use structopt::StructOpt;
use cmd::client;
use cmd::server;
use cmd::deployer;

/// A suite of programs for handling big omics data.
#[derive(StructOpt, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name = "Biopoem for DAG Task with Large-scale Servers.", author="Jingcheng Yang <yjcyxky@163.com>")]
struct Opt {
  /// A flag which control whether show more messages, true if used in the command line
  #[structopt(short="q", long="quiet")]
  quiet: bool,

  /// The number of occurrences of the `v/verbose` flag
  /// Verbose mode (-v, -vv, -vvv, etc.)
  #[structopt(short="v", long="verbose", parse(from_occurrences))]
  verbose: usize,

  #[structopt(subcommand)]
  cmd: SubCommands
}

#[derive(Debug, PartialEq, StructOpt)]
enum SubCommands {
  #[structopt(name="client")]
  Client(client::Arguments),
  #[structopt(name="server")]
  Server(server::Arguments),
  #[structopt(name="deployer")]
  Deployer(deployer::Arguments)
}

fn main() {
  let opt = Opt::from_args();

  match opt.cmd {
    SubCommands::Client(arguments) => {
      client::run(&arguments);
    },
    SubCommands::Server(arguments) => {
      server::run(&arguments);
    },
    SubCommands::Deployer(arguments) => {
      deployer::run(&arguments);
    }
  }
}
