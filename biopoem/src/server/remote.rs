use log::info;
use openssh::{Error, Session, SessionBuilder};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

pub async fn init_session(
  host: &str,
  port: u16,
  username: &str,
  keyfile: &PathBuf,
  remote_workdir: &str,
) -> Result<Session, Error> {
  let mut session = SessionBuilder::default();
  session
    .user(username.to_string())
    .port(port)
    .keyfile(Path::new(keyfile).to_path_buf())
    .control_directory(remote_workdir);

  return session.connect(host).await;
}

pub async fn init_env(session: &Session, remote_workdir: &str, dag: &PathBuf, biopoem_bin_url: &str) {
  let mut sftp = session.sftp();
  let mut w = sftp
    .write_to(format!("{}/{}", remote_workdir, "dag.factfile"))
    .await
    .unwrap();

  let bytes = std::fs::read_to_string(dag).unwrap();
  w.write_all(bytes.as_bytes()).await.unwrap();

  // flush and close the remote file, absorbing any final errors
  w.close().await.unwrap();

  info!("Download biopoem binary");
  let output = session
    .command(format!(
      "wget {} -O {}/{}",
      biopoem_bin_url, remote_workdir, "biopoem"
    ))
    .output()
    .await
    .unwrap();
  info!("{:?}", output);

  session
    .command(format!("chmod a+x {}/biopoem", remote_workdir))
    .output()
    .await
    .unwrap();
}

pub async fn launch_biopoem(
  session: &Session,
  remote_workdir: &str,
  webhook_url: &str,
  port: &str,
) {
  let biopoem_output = session
    .command(format!(
      "biopoem --workdir {} --host 0.0.0.0 --webhook {} --port {} --dag dag.factfile &",
      remote_workdir, webhook_url, port
    ))
    .output()
    .await
    .unwrap();
  info!("{:?}", biopoem_output);
}
