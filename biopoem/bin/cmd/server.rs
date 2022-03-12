use biopoem_api::{server, server::dag, server::remote};
use std::path::Path;
use std::path::PathBuf;
use std::{env, fs, process};
use structopt::StructOpt;
use super::init_file_logger;

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

#[tokio::main]
pub async fn run(args: &Arguments) {
  let workdir = &args.workdir;
  biopoem_api::makedir(workdir);

  let tmplpath = PathBuf::from(&args.dag_template);
  let dag_template = fs::canonicalize(tmplpath).unwrap();

  let varpath = PathBuf::from(&args.variable_file);
  let variable_file = fs::canonicalize(varpath).unwrap();

  let keypath = PathBuf::from(&args.keyfile);
  let keyfile = fs::canonicalize(keypath).unwrap();

  if let Err(log) = init_file_logger("Server", "server.log") {
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

  let hosts = server::host::read_hosts(&args.hosts);
  for host in &hosts {
    // Generate dag file.
    let hostname = host.hostname();
    let subdir = format!("results/{}", hostname);
    biopoem_api::makedir(&subdir);

    let destfile = Path::new(&subdir).join("dag.factfile");
    let template = fs::read_to_string(&dag_template).unwrap();

    info!("Rendering the dag template to {}", destfile.display());
    match dag::render_template(&template, &variable_file, hostname) {
      Some(result) => {
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
        match session.close().await {
          Err(msg) => warn!("{}", msg),
          _ => {}
        };
      }
      None => {
        error!(
          "Not found a context in {} with {}",
          variable_file.display(),
          hostname
        );
      }
    };
  }
}
