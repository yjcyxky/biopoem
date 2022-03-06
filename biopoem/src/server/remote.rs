use log::{error, info};
use openssh::{Error, KnownHosts, Session, SessionBuilder};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

pub async fn init_session(
  host: &str,
  port: u16,
  username: &str,
  keyfile: &PathBuf,
) -> Result<Session, Error> {
  let mut session = SessionBuilder::default();
  session
    .user(username.to_string())
    .port(port)
    .keyfile(keyfile)
    .known_hosts_check(KnownHosts::Accept)
    .control_directory("/tmp");

  info!(
    "Connect {} with {}(user) and {}(keyfile)",
    host,
    username,
    keyfile.display()
  );
  return session.connect(host).await;
}

pub async fn init_env(
  session: &Session,
  remote_workdir: &str,
  dag: &PathBuf,
  biopoem_bin_url: &str,
) {
  info!(
    "Create the working directory({}) on remote machine.",
    remote_workdir
  );
  let output = session
    .command("mkdir")
    .raw_arg(format!("-p {}", remote_workdir))
    .output()
    .await
    .unwrap();
  info!("{:?}", output);

  info!("Upload dag.factfile.");
  let mut sftp = session.sftp();
  let mut w = sftp
    .write_to(format!("{}/{}", remote_workdir, "dag.factfile"))
    .await
    .unwrap();

  let bytes = std::fs::read_to_string(dag).unwrap();
  w.write_all(bytes.as_bytes()).await.unwrap();

  // flush and close the remote file, absorbing any final errors
  w.close().await.unwrap();

  info!("Download biopoem binary.");
  match session
    .command("wget")
    .raw_arg(format!(
      "{} -O {}/{}",
      biopoem_bin_url, remote_workdir, "biopoem"
    ))
    .output()
    .await
  {
    Err(msg) => {
      error!("{:?}", msg);
    }
    Ok(output) => {
      info!("{:?}", output);
    }
  }

  match session
    .command("chmod")
    .raw_arg(format!("a+x {}/biopoem", remote_workdir))
    .output()
    .await
  {
    Err(msg) => {
      error!("{:?}", msg);
    }
    Ok(output) => {
      info!("{:?}", output);
    }
  }
}

pub async fn launch_biopoem(session: &Session, remote_workdir: &str, webhook_url: &str, port: u16) {
  info!("Launch biopoem...");
  // Why must need 2>&1? More details on https://askubuntu.com/a/1129702
  match session
    .command("nohup")
    .raw_arg(format!(
      "{}/biopoem client --workdir {} --host 0.0.0.0 --webhook {} --port {} --dag dag.factfile > {}/init.log 2>&1 &",
      remote_workdir, remote_workdir, webhook_url, port, remote_workdir
    ))
    .output()
    .await {
      Err(msg) => {
        error!("{:?}", msg);
      },
      Ok(output) => {
        info!("{:?}", output);
      }
    };
}
