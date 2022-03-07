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
  #[structopt(name = "region", short = "r", long = "region")]
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

fn notexists_exit(path: &PathBuf, msg: &str) {
  if !Path::exists(path.as_path()) {
    error!("{}", msg);
    process::exit(biopoem_api::PROC_OTHER_ERROR);
  }
}

fn exists_exit(path: &PathBuf, msg: &str) {
  if Path::exists(path.as_path()) {
    error!("{}", msg);
    process::exit(biopoem_api::PROC_OTHER_ERROR);
  }
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

  // Deploy servers by terraform
  let subdir = "terraform";
  biopoem_api::makedir(&subdir);

  if args.destroy {
    warn!("!!!Destroy Servers!!!");
    if let Some(destroy_output) = deployer::run(
      "destroy",
      subdir,
      &args.access_key,
      &args.secret_key,
      &args.region,
    ) {
      info!("Destroy Servers \n\n {}", destroy_output);
    }
  } else {
    let tmplpath = PathBuf::from(&args.template);
    notexists_exit(
      &tmplpath,
      &format!("Not found the file {}", tmplpath.display()),
    );
    let template = fs::canonicalize(tmplpath).unwrap();

    let destfile = Path::new(&subdir).join("terraform.tf");
    exists_exit(
      &destfile,
      &format!("The file {} exists!", destfile.display()),
    );
    let template = fs::read_to_string(&template).unwrap();
    let data = deployer::Config::new(
      &args.region,
      &args.zone,
      args.num_of_hosts,
      &args.image,
      &args.instance_type,
      "biopoem-secret-key",
    );

    info!("Set the current working directory to {}", &workdir);
    match env::set_current_dir(&workdir) {
      Err(msg) => {
        println!("Cannot set working directory {}.", &msg);
        process::exit(biopoem_api::PROC_OTHER_ERROR);
      }
      _ => {}
    };

    info!("Rendering the terraform template to {}", destfile.display());
    match deployer::render_template(&template, &data) {
      Some(result) => {
        fs::write(&destfile, result).unwrap();

        // Initialize Terraform
        if let Some(init_output) = deployer::run(
          "init",
          subdir,
          &args.access_key,
          &args.secret_key,
          &args.region,
        ) {
          info!("Initialize Terraform \n\n {}", init_output);
        }

        // Deploy Servers
        if let Some(apply_output) = deployer::run(
          "apply",
          subdir,
          &args.access_key,
          &args.secret_key,
          &args.region,
        ) {
          info!("Deploy Servers \n\n {}", apply_output);
        }

        // Get outputs
        if let Some(outputs) = deployer::run(
          "output",
          subdir,
          &args.access_key,
          &args.secret_key,
          &args.region,
        ) {
          info!("Output {}", outputs);
          let public_ips: Vec<String> = serde_json::from_str(&outputs).unwrap();

          info!("Generate hosts file");
          match fs::remove_file("hosts") {
            _ => {}
          };
          let hosts = deployer::gen_hosts(&data, &public_ips);
          println!("{:?}, {:?}", hosts, data);
          let mut wtr = csv::Writer::from_writer(fs::File::create("hosts").unwrap());
          for host in hosts {
            wtr.serialize(host).unwrap();
          }
          wtr.flush().unwrap();
        }
      }
      None => {}
    };
  }
}
