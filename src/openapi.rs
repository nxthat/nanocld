#[cfg(feature = "dev")]
use std::collections::HashMap;
#[cfg(feature = "dev")]
use serde::{Serialize, Deserialize, Deserializer};
#[cfg(feature = "dev")]
use serde::de::DeserializeOwned;
#[cfg(feature = "dev")]
use ntex::web;
#[cfg(feature = "dev")]
use utoipa::OpenApi;
#[cfg(feature = "dev")]
use utoipa::ToSchema;
#[cfg(feature = "dev")]
use crate::models::*;
#[cfg(feature = "dev")]
use crate::services::*;
#[cfg(feature = "dev")]
use crate::errors::ApiError;
#[cfg(feature = "dev")]
use ntex_files as fs;

#[cfg(feature = "dev")]
fn deserialize_nonoptional_vec<
  'de,
  D: Deserializer<'de>,
  T: DeserializeOwned,
>(
  d: D,
) -> Result<Vec<T>, D::Error> {
  serde::Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())
}

#[cfg(feature = "dev")]
fn deserialize_nonoptional_map<
  'de,
  D: Deserializer<'de>,
  T: DeserializeOwned,
>(
  d: D,
) -> Result<HashMap<String, T>, D::Error> {
  serde::Deserialize::deserialize(d).map(|x: Option<_>| x.unwrap_or_default())
}

#[cfg(feature = "dev")]
#[derive(
  Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema,
)]
pub struct ImageSummary {
  /// ID is the content-addressable ID of an image.  This identifier is a content-addressable digest calculated from the image's configuration (which includes the digests of layers used by the image).  Note that this digest differs from the `RepoDigests` below, which holds digests of image manifests that reference the image.
  #[serde(rename = "Id")]
  pub id: String,

  /// ID of the parent image.  Depending on how the image was created, this field may be empty and is only set for images that were built/created locally. This field is empty if the image was pulled from an image registry.
  #[serde(rename = "ParentId")]
  pub parent_id: String,

  /// List of image names/tags in the local image cache that reference this image.  Multiple image tags can refer to the same image, and this list may be empty if no tags reference the image, in which case the image is \"untagged\", in which case it can still be referenced by its ID.
  #[serde(rename = "RepoTags")]
  #[serde(deserialize_with = "deserialize_nonoptional_vec")]
  pub repo_tags: Vec<String>,

  /// List of content-addressable digests of locally available image manifests that the image is referenced from. Multiple manifests can refer to the same image.  These digests are usually only available if the image was either pulled from a registry, or if the image was pushed to a registry, which is when the manifest is generated and its digest calculated.
  #[serde(rename = "RepoDigests")]
  #[serde(deserialize_with = "deserialize_nonoptional_vec")]
  pub repo_digests: Vec<String>,

  /// Date and time at which the image was created as a Unix timestamp (number of seconds sinds EPOCH).
  #[serde(rename = "Created")]
  pub created: i64,

  /// Total size of the image including all layers it is composed of.
  #[serde(rename = "Size")]
  pub size: i64,

  /// Total size of image layers that are shared between this image and other images.  This size is not calculated by default. `-1` indicates that the value has not been set / calculated.
  #[serde(rename = "SharedSize")]
  pub shared_size: i64,

  /// Total size of the image including all layers it is composed of.  In versions of Docker before v1.10, this field was calculated from the image itself and all of its parent images. Docker v1.10 and up store images self-contained, and no longer use a parent-chain, making this field an equivalent of the Size field.  This field is kept for backward compatibility, but may be removed in a future version of the API.
  #[serde(rename = "VirtualSize")]
  pub virtual_size: i64,

  /// User-defined key/value metadata.
  #[serde(rename = "Labels")]
  #[serde(deserialize_with = "deserialize_nonoptional_map")]
  pub labels: HashMap<String, String>,

  /// Number of containers using this image. Includes both stopped and running containers.  This size is not calculated by default, and depends on which API endpoint is used. `-1` indicates that the value has not been set / calculated.
  #[serde(rename = "Containers")]
  pub containers: i64,
}

#[cfg(feature = "dev")]
/// Information about an image in the local image cache.
#[derive(
  Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema,
)]
pub struct ImageInspect {
  /// ID is the content-addressable ID of an image.  This identifier is a content-addressable digest calculated from the image's configuration (which includes the digests of layers used by the image).  Note that this digest differs from the `RepoDigests` below, which holds digests of image manifests that reference the image.
  #[serde(rename = "Id")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,

  /// List of image names/tags in the local image cache that reference this image.  Multiple image tags can refer to the same image, and this list may be empty if no tags reference the image, in which case the image is \"untagged\", in which case it can still be referenced by its ID.
  #[serde(rename = "RepoTags")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub repo_tags: Option<Vec<String>>,

  /// List of content-addressable digests of locally available image manifests that the image is referenced from. Multiple manifests can refer to the same image.  These digests are usually only available if the image was either pulled from a registry, or if the image was pushed to a registry, which is when the manifest is generated and its digest calculated.
  #[serde(rename = "RepoDigests")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub repo_digests: Option<Vec<String>>,

  /// ID of the parent image.  Depending on how the image was created, this field may be empty and is only set for images that were built/created locally. This field is empty if the image was pulled from an image registry.
  #[serde(rename = "Parent")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub parent: Option<String>,

  /// Optional message that was set when committing or importing the image.
  #[serde(rename = "Comment")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub comment: Option<String>,

  /// Date and time at which the image was created, formatted in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.
  #[serde(rename = "Created")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub created: Option<String>,

  /// The ID of the container that was used to create the image.  Depending on how the image was created, this field may be empty.
  #[serde(rename = "Container")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub container: Option<String>,

  #[serde(rename = "ContainerConfig")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub container_config: Option<ContainerConfig>,

  /// The version of Docker that was used to build the image.  Depending on how the image was created, this field may be empty.
  #[serde(rename = "DockerVersion")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub docker_version: Option<String>,

  /// Name of the author that was specified when committing the image, or as specified through MAINTAINER (deprecated) in the Dockerfile.
  #[serde(rename = "Author")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub author: Option<String>,

  #[serde(rename = "Config")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub config: Option<ContainerConfig>,

  /// Hardware CPU architecture that the image runs on.
  #[serde(rename = "Architecture")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub architecture: Option<String>,

  /// CPU architecture variant (presently ARM-only).
  #[serde(rename = "Variant")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub variant: Option<String>,

  /// Operating System the image is built to run on.
  #[serde(rename = "Os")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub os: Option<String>,

  /// Operating System version the image is built to run on (especially for Windows).
  #[serde(rename = "OsVersion")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub os_version: Option<String>,

  /// Total size of the image including all layers it is composed of.
  #[serde(rename = "Size")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub size: Option<i64>,

  /// Total size of the image including all layers it is composed of.  In versions of Docker before v1.10, this field was calculated from the image itself and all of its parent images. Docker v1.10 and up store images self-contained, and no longer use a parent-chain, making this field an equivalent of the Size field.  This field is kept for backward compatibility, but may be removed in a future version of the API.
  #[serde(rename = "VirtualSize")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub virtual_size: Option<i64>,

  #[serde(rename = "GraphDriver")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub graph_driver: Option<GraphDriverData>,

  #[serde(rename = "RootFS")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub root_fs: Option<ImageInspectRootFs>,

  #[serde(rename = "Metadata")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub metadata: Option<ImageInspectMetadata>,
}

/// Information about the storage driver used to store the container's and image's filesystem.
#[cfg(feature = "dev")]
#[derive(
  Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema,
)]
pub struct GraphDriverData {
  /// Name of the storage driver.
  #[serde(rename = "Name")]
  pub name: String,

  /// Low-level storage metadata, provided as key/value pairs.  This information is driver-specific, and depends on the storage-driver in use, and should be used for informational purposes only.
  #[serde(rename = "Data")]
  #[serde(deserialize_with = "deserialize_nonoptional_map")]
  pub data: HashMap<String, String>,
}

/// Information about the image's RootFS, including the layer IDs.
#[cfg(feature = "dev")]
#[derive(
  Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema,
)]
pub struct ImageInspectRootFs {
  #[serde(rename = "Type")]
  pub typ: String,

  #[serde(rename = "Layers")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub layers: Option<Vec<String>>,
}

/// Additional metadata of the image in the local cache. This information is local to the daemon, and not part of the image itself.
#[cfg(feature = "dev")]
#[derive(
  Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema,
)]
pub struct ImageInspectMetadata {
  /// Date and time at which the image was last tagged in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt) format with nano-seconds.  This information is only available if the image was tagged locally, and omitted otherwise.
  #[serde(rename = "LastTagTime")]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(default)]
  pub last_tag_time: Option<String>,
}

/// Configuration for a container that is portable between hosts.  When used as `ContainerConfig` field in an image, `ContainerConfig` is an optional field containing the configuration of the container that was last committed when creating the image.  Previous versions of Docker builder used this field to store build cache, and it is not in active use anymore.
#[cfg(feature = "dev")]
#[derive(
  Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema,
)]
pub struct ContainerConfig {
  /// The hostname to use for the container, as a valid RFC 1123 hostname.
  #[serde(rename = "Hostname")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub hostname: Option<String>,

  /// The domain name to use for the container.
  #[serde(rename = "Domainname")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub domainname: Option<String>,

  /// The user that commands are run as inside the container.
  #[serde(rename = "User")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user: Option<String>,

  /// Whether to attach to `stdin`.
  #[serde(rename = "AttachStdin")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub attach_stdin: Option<bool>,

  /// Whether to attach to `stdout`.
  #[serde(rename = "AttachStdout")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub attach_stdout: Option<bool>,

  /// Whether to attach to `stderr`.
  #[serde(rename = "AttachStderr")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub attach_stderr: Option<bool>,

  /// An object mapping ports to an empty object in the form:  `{\"<port>/<tcp|udp|sctp>\": {}}`
  // #[serde(rename = "ExposedPorts")]
  // #[serde(skip_serializing_if = "Option::is_none")]
  // pub exposed_ports: Option<HashMap<String, HashMap<(), ()>>>,

  /// Attach standard streams to a TTY, including `stdin` if it is not closed.
  #[serde(rename = "Tty")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tty: Option<bool>,

  /// Open `stdin`
  #[serde(rename = "OpenStdin")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub open_stdin: Option<bool>,

  /// Close `stdin` after one attached client disconnects
  #[serde(rename = "StdinOnce")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stdin_once: Option<bool>,

  /// A list of environment variables to set inside the container in the form `[\"VAR=value\", ...]`. A variable without `=` is removed from the environment, rather than to have an empty value.
  #[serde(rename = "Env")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub env: Option<Vec<String>>,

  /// Command to run specified as a string or an array of strings.
  #[serde(rename = "Cmd")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub cmd: Option<Vec<String>>,

  #[serde(rename = "Healthcheck")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub healthcheck: Option<HealthConfig>,

  /// Command is already escaped (Windows only)
  #[serde(rename = "ArgsEscaped")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub args_escaped: Option<bool>,

  /// The name (or reference) of the image to use when creating the container, or which was used when the container was created.
  #[serde(rename = "Image")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub image: Option<String>,

  /// An object mapping mount point paths inside the container to empty objects.
  // #[serde(rename = "Volumes")]
  // #[serde(skip_serializing_if = "Option::is_none")]
  // pub volumes: Option<HashMap<String, HashMap<(), ()>>>,

  /// The working directory for commands to run in.
  #[serde(rename = "WorkingDir")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub working_dir: Option<String>,

  /// The entry point for the container as a string or an array of strings.  If the array consists of exactly one empty string (`[\"\"]`) then the entry point is reset to system default (i.e., the entry point used by docker when there is no `ENTRYPOINT` instruction in the `Dockerfile`).
  #[serde(rename = "Entrypoint")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub entrypoint: Option<Vec<String>>,

  /// Disable networking for the container.
  #[serde(rename = "NetworkDisabled")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub network_disabled: Option<bool>,

  /// MAC address of the container.
  #[serde(rename = "MacAddress")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mac_address: Option<String>,

  /// `ONBUILD` metadata that were defined in the image's `Dockerfile`.
  #[serde(rename = "OnBuild")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub on_build: Option<Vec<String>>,

  /// User-defined key/value metadata.
  // #[serde(rename = "Labels")]
  // #[serde(skip_serializing_if = "Option::is_none")]
  // pub labels: Option<HashMap<String, String>>,

  /// Signal to stop a container as a string or unsigned integer.
  #[serde(rename = "StopSignal")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stop_signal: Option<String>,

  /// Timeout to stop a container in seconds.
  #[serde(rename = "StopTimeout")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stop_timeout: Option<i64>,

  /// Shell for when `RUN`, `CMD`, and `ENTRYPOINT` uses a shell.
  #[serde(rename = "Shell")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub shell: Option<Vec<String>>,
}

/// A test to perform to check that the container is healthy.
#[cfg(feature = "dev")]
#[derive(
  Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema,
)]
pub struct HealthConfig {
  /// The test to perform. Possible values are:  - `[]` inherit healthcheck from image or parent image - `[\"NONE\"]` disable healthcheck - `[\"CMD\", args...]` exec arguments directly - `[\"CMD-SHELL\", command]` run command with system's default shell
  #[serde(rename = "Test")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub test: Option<Vec<String>>,

  /// The time to wait between checks in nanoseconds. It should be 0 or at least 1000000 (1 ms). 0 means inherit.
  #[serde(rename = "Interval")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub interval: Option<i64>,

  /// The time to wait before considering the check to have hung. It should be 0 or at least 1000000 (1 ms). 0 means inherit.
  #[serde(rename = "Timeout")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub timeout: Option<i64>,

  /// The number of consecutive failures needed to consider a container as unhealthy. 0 means inherit.
  #[serde(rename = "Retries")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub retries: Option<i64>,

  /// Start period for the container to initialize before starting health-retries countdown in nanoseconds. It should be 0 or at least 1000000 (1 ms). 0 means inherit.
  #[serde(rename = "StartPeriod")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub start_period: Option<i64>,
}

#[cfg_attr(feature = "dev", derive(OpenApi))]
#[cfg_attr(feature = "dev", openapi(
  paths(
    // Namespace
    namespace::list_namespace,
    namespace::create_namespace,
    namespace::delete_namespace_by_name,
    namespace::inspect_namespace_by_name,

    // proxy template
    proxy_template::list_proxy_template,

    // Cargo images
    cargo_image::list_cargo_image,
    cargo_image::create_cargo_image,
    cargo_image::inspect_cargo_image,
    cargo_image::delete_cargo_image_by_name,

    // Cargo
    cargo::list_cargo,
    cargo::create_cargo,
    cargo::delete_cargo_by_name,
    cargo::count_cargo,

    // Cluster
    cluster::list_cluster,
    cluster::count_cluster,
    cluster::create_cluster,
    cluster::delete_cluster_by_name,
    cluster::inspect_cluster_by_name,
    cluster::start_cluster_by_name,
    cluster::join_cargo_to_cluster,

    // Cluster variable
    cluster_variable::list_cluster_variable,
    cluster_variable::create_cluster_variable,
    cluster_variable::delete_cluster_variable,

    // Cluster network
    cluster_network::list_cluster_network,
    cluster_network::create_cluster_network,
    cluster_network::delete_cluster_network_by_name,
    cluster_network::inspect_cluster_network_by_name,
    cluster_network::count_cluster_network_by_namespace,
  ),
  components(
    schemas(ApiError),
    schemas(GenericDelete),
    schemas(GenericCount),

    // Proxy template
    schemas(ProxyTemplateItem),
    schemas(ProxyTemplateModes),

    // Namespace
    schemas(NamespaceItem),
    schemas(NamespacePartial),

    // Cargo
    schemas(CargoItem),
    schemas(CargoPartial),

    // Cluster
    schemas(ClusterItem),
    schemas(ClusterPartial),
    schemas(ClusterJoinBody),

    // Cluster variable
    schemas(ClusterVariableItem),
    schemas(ClusterVariablePartial),
    schemas(ClusterItemWithRelation),

    // Cluster network
    schemas(ClusterNetworkItem),
    schemas(ClusterNetworkPartial),

    // Cargo images
    schemas(ImageSummary),
    schemas(ImageInspect),
    schemas(ContainerConfig),
    schemas(GraphDriverData),
    schemas(ImageInspectMetadata),
    schemas(ImageInspectRootFs),
    schemas(HealthConfig),
    schemas(CargoImagePartial),

    // ClusterItemWithRelation,

    // Todo Docker network struct bindings
    // Network,
    // Ipam,
    // IpamConfig,
    // NetworkContainer,
  )
))]
#[cfg(feature = "dev")]
pub struct ApiDoc;

#[cfg(feature = "dev")]
pub fn to_json() -> String {
  ApiDoc::openapi().to_pretty_json().unwrap()
}

#[cfg(feature = "dev")]
#[web::get("/explorer/swagger.json")]
async fn get_api_specs() -> Result<web::HttpResponse, web::Error> {
  let api_spec = to_json();
  return Ok(
    web::HttpResponse::Ok()
      .header("Access-Control-Allow", "*")
      .content_type("application/json")
      .body(api_spec),
  );
}

#[cfg(feature = "dev")]
pub fn ntex_config(config: &mut web::ServiceConfig) {
  config.service(get_api_specs);
  config.service(
    // static files
    fs::Files::new("/explorer", "./swagger-ui/").index_file("index.html"),
  );
}
