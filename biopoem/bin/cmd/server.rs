use biopoem_api::{server, server::dag, server::remote};
use log::LevelFilter;
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::error::Error;
use std::path::Path;
use std::path::PathBuf;
use std::{env, fs, process};
use structopt::StructOpt;

/// Server for Biopoem
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="Biopoem - Server", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct Arguments {
  /// Which working directory for saving data.
  #[structopt(name = "workdir", short = "w", long = "workdir", default_value = ".")]
  workdir: String,

  /// The host file.
  #[structopt(name = "hosts", short = "-H", long = "hosts", default_value = "hosts")]
  hosts: String,

  /// The template file for DAG.
  #[structopt(
    name = "dag-template",
    short = "t",
    long = "dag-template",
    default_value = "dag.template"
  )]
  dag_template: String,

  /// The variable file for DAG (json).
  #[structopt(
    name = "variable-file",
    short = "f",
    long = "variable-file",
    default_value = "variables"
  )]
  variable_file: String,

  /// The private key file for ssh (such as .ssh/id_rsa).
  #[structopt(
    name = "keyfile",
    short = "k",
    long = "keyfile",
    default_value = "keyfile"
  )]
  keyfile: String,

  /// The working directory on remote machine.
  #[structopt(
    name = "remote-workdir",
    short = "r",
    long = "remote-workdir",
    default_value = "/mnt/biopoem"
  )]
  remote_workdir: String,
}

fn init_logger() -> Result<log4rs::Handle, String> {
  let stdout = ConsoleAppender::builder()
    .encoder(Box::new(PatternEncoder::new(
      "[Server] {d} - {l} -{t} - {m}{n}",
    )))
    .build();

  match fs::remove_file("server.log") {
    _ => {}
  };

  let config = Config::builder()
    .appender(Appender::builder().build("stdout", Box::new(stdout)))
    .logger(
      Logger::builder()
        .appender("stdout")
        .additive(false)
        .build("stdout", LevelFilter::Info),
    )
    .build(Root::builder().appender("stdout").build(LevelFilter::Info))
    .unwrap();

  log4rs::init_config(config).map_err(|e| {
    format!(
      "couldn't initialize log configuration. Reason: {}",
      e.description()
    )
  })
}

fn makedir(dir: &str) {
  if !Path::new(&dir).exists() {
    match fs::create_dir_all(dir) {
      Err(err) => {
        println!("Cannot create the directory {}, {}", dir, err);
        process::exit(biopoem_api::PROC_OTHER_ERROR);
      }
      Ok(_) => {
        println!("Create the directory {}", dir);
      }
    };
  }
}

#[tokio::main]
pub async fn run(args: &Arguments) {
  let workdir = &args.workdir;
  makedir(workdir);

  let tmplpath = PathBuf::from(&args.dag_template);
  let dag_template = fs::canonicalize(tmplpath).unwrap();

  let varpath = PathBuf::from(&args.variable_file);
  let variable_file = fs::canonicalize(varpath).unwrap();

  let keypath = PathBuf::from(&args.keyfile);
  let keyfile = fs::canonicalize(keypath).unwrap();

  if let Err(log) = init_logger() {
    error!(target:"stdout", "Log initialization error, {}", log);
    process::exit(biopoem_api::PROC_OTHER_ERROR);
  };

  info!("Set the current working directory to {}", &workdir);
  match env::set_current_dir(&workdir) {
    Err(msg) => {
      println!("Cannot set working directory {}.", &msg);
      process::exit(biopoem_api::PROC_OTHER_ERROR);
    }
    _ => {}
  };

  // Deploy servers by terraform

  // Save hosts into working directory

  let hosts = server::host::read_hosts(&args.hosts);
  for host in &hosts {
    // Generate dag file.
    let subdir = format!("results/{}", &host.hostname());
    makedir(&subdir);

    let destfile = Path::new(&subdir).join("dag.factfile");
    let template = fs::read_to_string(&dag_template).unwrap();

    info!("Rendering the dag template to {}", destfile.display());
    let result = dag::render_template(&template, &variable_file);
    fs::write(&destfile, result).unwrap();

    // Initialize (Upload biopoem and dag file.)
    let port = host.port().parse().unwrap();
    let remote_workdir = &args.remote_workdir;
    let biopoem_bin_url = "http://nordata-cdn.oss-cn-shanghai.aliyuncs.com/biopoem/biopoem";
    let session = remote::init_session(host.ipaddr(), port, host.username(), &keyfile)
      .await
      .unwrap();

    remote::init_env(&session, remote_workdir, &destfile, biopoem_bin_url).await;
    remote::launch_biopoem(&session, remote_workdir, "", 3000).await;
    session.close().await.unwrap();
  }

  // Get logs periodically
}
