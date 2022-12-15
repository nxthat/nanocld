/// Models specification for utoipa used to generate OpenAPI specification
/// This is enabled only when the `dev` feature is set.
#[cfg(feature = "dev")]
use std::collections::HashMap;
#[cfg(feature = "dev")]
use serde::{Serialize, Deserialize, Deserializer};
#[cfg(feature = "dev")]
use serde::de::DeserializeOwned;
#[cfg(feature = "dev")]
use utoipa::ToSchema;

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

#[cfg(feature = "dev")]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema,
)]
pub struct ContainerSummary {
  /// The ID of this container
  #[serde(rename = "Id")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,

  /// The names that this container has been given
  #[serde(rename = "Names")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub names: Option<Vec<String>>,

  /// The name of the image used when creating this container
  #[serde(rename = "Image")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub image: Option<String>,

  /// The ID of the image that this container was created from
  #[serde(rename = "ImageID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub image_id: Option<String>,

  /// Command to run when starting the container
  #[serde(rename = "Command")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub command: Option<String>,

  /// When the container was created
  #[serde(rename = "Created")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub created: Option<i64>,

  /// The ports exposed by this container
  #[serde(rename = "Ports")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ports: Option<Vec<Port>>,

  /// The size of files that have been created or changed by this container
  #[serde(rename = "SizeRw")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub size_rw: Option<i64>,

  /// The total size of all the files in this container
  #[serde(rename = "SizeRootFs")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub size_root_fs: Option<i64>,

  /// User-defined key/value metadata.
  #[serde(rename = "Labels")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub labels: Option<HashMap<String, String>>,

  /// The state of this container (e.g. `Exited`)
  #[serde(rename = "State")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub state: Option<String>,

  /// Additional human-readable status of this container (e.g. `Exit 0`)
  #[serde(rename = "Status")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub status: Option<String>,

  #[serde(rename = "HostConfig")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub host_config: Option<ContainerSummaryHostConfig>,

  #[serde(rename = "NetworkSettings")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub network_settings: Option<ContainerSummaryNetworkSettings>,

  #[serde(rename = "Mounts")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mounts: Option<Vec<MountPoint>>,
}

#[cfg(feature = "dev")]
#[derive(
  Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema,
)]
pub struct ContainerSummaryHostConfig {
  #[serde(rename = "NetworkMode")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub network_mode: Option<String>,
}

/// A summary of the container's network settings
#[cfg(feature = "dev")]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema,
)]
pub struct ContainerSummaryNetworkSettings {
  #[serde(rename = "Networks")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub networks: Option<HashMap<String, EndpointSettings>>,
}

/// MountPoint represents a mount point configuration inside the container. This is used for reporting the mountpoints in use by a container.
#[cfg(feature = "dev")]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema,
)]
pub struct MountPoint {
  /// The mount type:  - `bind` a mount of a file or directory from the host into the container. - `volume` a docker volume with the given `Name`. - `tmpfs` a `tmpfs`. - `npipe` a named pipe from the host into the container.
  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub typ: Option<MountPointTypeEnum>,

  /// Name is the name reference to the underlying data defined by `Source` e.g., the volume name.
  #[serde(rename = "Name")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,

  /// Source location of the mount.  For volumes, this contains the storage location of the volume (within `/var/lib/docker/volumes/`). For bind-mounts, and `npipe`, this contains the source (host) part of the bind-mount. For `tmpfs` mount points, this field is empty.
  #[serde(rename = "Source")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source: Option<String>,

  /// Destination is the path relative to the container root (`/`) where the `Source` is mounted inside the container.
  #[serde(rename = "Destination")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub destination: Option<String>,

  /// Driver is the volume driver used to create the volume (if it is a volume).
  #[serde(rename = "Driver")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub driver: Option<String>,

  /// Mode is a comma separated list of options supplied by the user when creating the bind/volume mount.  The default is platform-specific (`\"z\"` on Linux, empty on Windows).
  #[serde(rename = "Mode")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mode: Option<String>,

  /// Whether the mount is mounted writable (read-write).
  #[serde(rename = "RW")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rw: Option<bool>,

  /// Propagation describes how mounts are propagated from the host into the mount point, and vice-versa. Refer to the [Linux kernel documentation](https://www.kernel.org/doc/Documentation/filesystems/sharedsubtree.txt) for details. This field is not used on Windows.
  #[serde(rename = "Propagation")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub propagation: Option<String>,
}

#[cfg(feature = "dev")]
#[allow(non_camel_case_types)]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  PartialOrd,
  Serialize,
  Deserialize,
  Eq,
  Ord,
  ToSchema,
)]
pub enum MountPointTypeEnum {
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "bind")]
  BIND,
  #[serde(rename = "volume")]
  VOLUME,
  #[serde(rename = "tmpfs")]
  TMPFS,
  #[serde(rename = "npipe")]
  NPIPE,
}

/// Configuration for a network endpoint.
#[cfg(feature = "dev")]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema,
)]
pub struct EndpointSettings {
  #[serde(rename = "IPAMConfig")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ipam_config: Option<EndpointIpamConfig>,

  #[serde(rename = "Links")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub links: Option<Vec<String>>,

  #[serde(rename = "Aliases")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub aliases: Option<Vec<String>>,

  /// Unique ID of the network.
  #[serde(rename = "NetworkID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub network_id: Option<String>,

  /// Unique ID for the service endpoint in a Sandbox.
  #[serde(rename = "EndpointID")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub endpoint_id: Option<String>,

  /// Gateway address for this network.
  #[serde(rename = "Gateway")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub gateway: Option<String>,

  /// IPv4 address.
  #[serde(rename = "IPAddress")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ip_address: Option<String>,

  /// Mask length of the IPv4 address.
  #[serde(rename = "IPPrefixLen")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ip_prefix_len: Option<i64>,

  /// IPv6 gateway address.
  #[serde(rename = "IPv6Gateway")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ipv6_gateway: Option<String>,

  /// Global IPv6 address.
  #[serde(rename = "GlobalIPv6Address")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub global_ipv6_address: Option<String>,

  /// Mask length of the global IPv6 address.
  #[serde(rename = "GlobalIPv6PrefixLen")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub global_ipv6_prefix_len: Option<i64>,

  /// MAC address for the endpoint on this network.
  #[serde(rename = "MacAddress")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub mac_address: Option<String>,

  /// DriverOpts is a mapping of driver options and values. These options are passed directly to the driver and are driver specific.
  #[serde(rename = "DriverOpts")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub driver_opts: Option<HashMap<String, String>>,
}

/// EndpointIPAMConfig represents an endpoint's IPAM configuration.
#[cfg(feature = "dev")]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema,
)]
pub struct EndpointIpamConfig {
  #[serde(rename = "IPv4Address")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ipv4_address: Option<String>,

  #[serde(rename = "IPv6Address")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ipv6_address: Option<String>,

  #[serde(rename = "LinkLocalIPs")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub link_local_i_ps: Option<Vec<String>>,
}

/// An open port on a container
#[cfg(feature = "dev")]
#[derive(
  Debug, Clone, Default, PartialEq, Serialize, Deserialize, ToSchema,
)]
pub struct Port {
  /// Host IP address that the container's port is mapped to
  #[serde(rename = "IP")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub ip: Option<String>,

  /// Port on the container
  #[serde(rename = "PrivatePort")]
  pub private_port: i64,

  /// Port exposed on the host
  #[serde(rename = "PublicPort")]
  #[serde(skip_serializing_if = "Option::is_none")]
  pub public_port: Option<i64>,

  #[serde(rename = "Type")]
  #[serde(skip_serializing_if = "Option::is_none")]
  #[serde(with = "::serde_with::As::<::serde_with::NoneAsEmptyString>")]
  pub typ: Option<PortTypeEnum>,
}

#[cfg(feature = "dev")]
#[allow(non_camel_case_types)]
#[derive(
  Debug,
  Clone,
  Copy,
  PartialEq,
  PartialOrd,
  Serialize,
  Deserialize,
  Eq,
  Ord,
  ToSchema,
)]
pub enum PortTypeEnum {
  #[serde(rename = "")]
  EMPTY,
  #[serde(rename = "tcp")]
  TCP,
  #[serde(rename = "udp")]
  UDP,
  #[serde(rename = "sctp")]
  SCTP,
}

impl ::std::fmt::Display for PortTypeEnum {
  fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    match *self {
      PortTypeEnum::EMPTY => write!(f, ""),
      PortTypeEnum::TCP => write!(f, "tcp"),
      PortTypeEnum::UDP => write!(f, "udp"),
      PortTypeEnum::SCTP => write!(f, "sctp"),
    }
  }
}

impl ::std::str::FromStr for PortTypeEnum {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "" => Ok(PortTypeEnum::EMPTY),
      "tcp" => Ok(PortTypeEnum::TCP),
      "udp" => Ok(PortTypeEnum::UDP),
      "sctp" => Ok(PortTypeEnum::SCTP),
      x => Err(format!("Invalid enum type: {}", x)),
    }
  }
}

impl ::std::convert::AsRef<str> for PortTypeEnum {
  fn as_ref(&self) -> &str {
    match self {
      PortTypeEnum::EMPTY => "",
      PortTypeEnum::TCP => "tcp",
      PortTypeEnum::UDP => "udp",
      PortTypeEnum::SCTP => "sctp",
    }
  }
}
