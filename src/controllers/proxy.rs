use bollard::{
  Docker,
  errors::Error as DockerError,
  exec::{CreateExecOptions, StartExecOptions},
};

pub async fn reload_config(docker_api: &Docker) -> Result<(), DockerError> {
  let container_name = "nproxy";
  let config = CreateExecOptions {
    cmd: Some(vec!["nginx", "-s", "reload"]),
    attach_stdout: Some(true),
    attach_stderr: Some(true),
    ..Default::default()
  };
  let res = docker_api.create_exec(container_name, config).await?;
  let config = StartExecOptions {
    detach: false,
    ..Default::default()
  };
  docker_api.start_exec(&res.id, Some(config)).await?;

  Ok(())
}
