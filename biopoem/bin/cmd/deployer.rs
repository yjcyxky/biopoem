use biopoem_api::deployer;
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

/// Deployer for Biopoem
#[derive(StructOpt, PartialEq, Debug)]
#[structopt(setting=structopt::clap::AppSettings::ColoredHelp, name="Biopoem - Deployer", author="Jingcheng Yang <yjcyxky@163.com>")]
pub struct Arguments {
  /// Which working directory for saving data.
  #[structopt(name = "workdir", short = "w", long = "workdir", default_value = ".")]
  workdir: String,

  /// How many hosts.
  #[structopt(
    name = "num-of-hosts",
    short = "-n",
    long = "num-of-hosts",
    default_value = "1"
  )]
  num_of_hosts: usize,

  /// The template file for deployment.
  #[structopt(
    name = "template",
    short = "t",
    long = "template",
    default_value = "template.tf"
  )]
  template: String,

  /// Region, such as cn-shanghai
  #[structopt(
    name = "region",
    short = "r",
    long = "region"
  )]
  region: String,

  /// Available Zone
  #[structopt(name = "zone", short = "z", long = "zone", default_value = "a")]
  zone: String,

  /// Instance Type
  #[structopt(
    name = "instance-type",
    short = "i",
    long = "instance-type",
    default_value = "ecs.t6-c2m1.large"
  )]
  instance_type: String,

  /// Image
  #[structopt(
    name = "image",
    short = "I",
    long = "image",
    default_value = "ubuntu_20_04_x64_20G_alibase_20220215.vhd"
  )]
  image: String,

  /// AccessKey.
  #[structopt(name = "access-key", short = "k", long = "access-key")]
  access_key: String,

  /// SecretKey.
  #[structopt(name = "secret-key", short = "s", long = "secret-key")]
  secret_key: String,

  /// Activate destroy mode.
  #[structopt(name = "destroy", short = "d", long = "destroy")]
  destroy: bool,
}

fn init_logger() -> Result<log4rs::Handle, String> {
  let stdout = ConsoleAppender::builder()
    .encoder(Box::new(PatternEncoder::new(
      "[Deployment] {d} - {l} -{t} - {m}{n}",
    )))
    .build();

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

#[tokio::main]
pub async fn run(args: &Arguments) {
  let workdir = &args.workdir;
  biopoem_api::makedir(workdir);

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
  let subdir = "terraform";
  biopoem_api::makedir(&subdir);

  if args.destroy {
    if let Some(destroy_output) =
      deployer::run("destroy", &args.access_key, &args.secret_key, &args.region)
    {
      info!("Destroy Servers \n\n {}", destroy_output);
    }
  } else {
    let tmplpath = PathBuf::from(&args.template);
    let template = fs::canonicalize(tmplpath).unwrap();

    let destfile = Path::new(&subdir).join("terraform.tf");
    let template = fs::read_to_string(&template).unwrap();
    let data = deployer::Config::new(
      &args.region,
      &args.zone,
      args.num_of_hosts,
      &args.image,
      &args.instance_type,
      "biopoem-secret-key",
    );

    info!("Rendering the terraform template to {}", destfile.display());
    match deployer::render_template(&template, &data) {
      Some(result) => {
        fs::write(&destfile, result).unwrap();
        if let Some(init_output) =
          deployer::run("init", &args.access_key, &args.secret_key, &args.region)
        {
          info!("Initialize Terraform \n\n {}", init_output);
        }

        if let Some(apply_output) =
          deployer::run("apply", &args.access_key, &args.secret_key, &args.region)
        {
          info!("Deploy Servers \n\n {}", apply_output);
        }
      }
      None => {}
    };
  }
}
