use super::formats::*;
use crate::infra::formats::NetworkId;
use crate::resources::formats::OrganizationId;
use crate::resources::formats::ProjectId;
use chrono::DateTime;
use chrono::Utc;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Acl {
    ResourceIdentifier(ResourceIdentifier),
    CreateAclRequest(CreateAclRequest),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AclCidrBlock {
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub available_node_count: i32,
    pub created: DateTime<Utc>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linked_resource: Option<String>,
    pub id: BackupId,
    pub project_id: ProjectId,
    pub provider: String,
    pub region: String,
    pub server_version: String,
    pub server_version_tag: String,
    pub size_gb: i32,
    pub source_cluster_id: ClusterId,
    pub source_cluster_description: String,
    pub status: BackupStatus,
}

/// The status of the cluster
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BackupStatus {
    Creating,
    Available,
    Deleted,
    Deleting,
    Defunct,
}
impl std::fmt::Display for BackupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupStatus::Creating => write!(f, "creating"),
            BackupStatus::Available => write!(f, "available"),
            BackupStatus::Deleted => write!(f, "deleted"),
            BackupStatus::Deleting => write!(f, "deleting"),
            BackupStatus::Defunct => write!(f, "defunct"),
        }
    }
}
impl std::cmp::PartialEq<&str> for BackupStatus {
    fn eq(&self, other: &&str) -> bool {
        match self {
            BackupStatus::Creating => *other == "creating",
            BackupStatus::Available => *other == "available",
            BackupStatus::Deleted => *other == "deleted",
            BackupStatus::Deleting => *other == "deleting",
            BackupStatus::Defunct => *other == "defunct",
        }
    }
}
impl std::cmp::PartialEq<BackupStatus> for &str {
    fn eq(&self, other: &BackupStatus) -> bool {
        other == self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cluster {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acl_id: Option<String>,
    pub can_expand_disk: bool,
    pub cloud_integrated_authentication: bool,
    pub created: DateTime<Utc>,
    pub description: String,
    pub disk_size_gb: i32,
    pub disk_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_iops: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_throughput: Option<i32>,
    pub id: ClusterId,
    pub instance_type: String,
    pub health: Health,
    pub network_id: NetworkId,
    pub organization_id: OrganizationId,
    pub patch_available: bool,
    pub project_id: ProjectId,
    pub projection_level: ProjectionLevel,
    pub protected: bool,
    pub provider: String,
    pub public_access: bool,
    pub region: String,
    pub server_version: String,
    pub server_version_tag: String,
    pub status: ClusterStatus,
    pub topology: Topology,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterCreateVersion {
    pub name: String,
    pub lts: bool,
    pub recommended: bool,
    pub tag: String,
    pub version: String,
}

/// The status of the cluster
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ClusterStatus {
    #[serde(rename = "provisioning")]
    Provisioning,
    #[serde(rename = "disks available")]
    DisksAvailable,
    #[serde(rename = "expanding disks")]
    ExpandingDisks,
    #[serde(rename = "restarting")]
    Restarting,
    #[serde(rename = "available")]
    Available,
    #[serde(rename = "defunct")]
    Defunct,
    #[serde(rename = "inconsistent")]
    Inconsistent,
    #[serde(rename = "upgrading")]
    Upgrading,
    #[serde(rename = "deleting instances")]
    DeletingInstances,
    #[serde(rename = "instances deleted")]
    InstancesDeleted,
    #[serde(rename = "deleting disks")]
    DeletingDisks,
    #[serde(rename = "deleted")]
    Deleted,
    #[serde(rename = "resizing")]
    Resizing,
    #[serde(rename = "stopping")]
    Stopping,
    #[serde(rename = "starting")]
    Starting,
    #[serde(rename = "updating configuration")]
    UpdatingConfiguration,
    #[serde(rename = "compute available")]
    ComputeAvailable,
    #[serde(rename = "installing")]
    Installing,
    #[serde(rename = "deploying")]
    Deploying,
    #[serde(rename = "updating")]
    Updating,
    #[serde(rename = "deleting")]
    Deleting,
}
impl std::fmt::Display for ClusterStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClusterStatus::Provisioning => write!(f, "provisioning"),
            ClusterStatus::DisksAvailable => write!(f, "disks available"),
            ClusterStatus::ExpandingDisks => write!(f, "expanding disks"),
            ClusterStatus::Restarting => write!(f, "restarting"),
            ClusterStatus::Available => write!(f, "available"),
            ClusterStatus::Defunct => write!(f, "defunct"),
            ClusterStatus::Inconsistent => write!(f, "inconsistent"),
            ClusterStatus::Upgrading => write!(f, "upgrading"),
            ClusterStatus::DeletingInstances => write!(f, "deleting instances"),
            ClusterStatus::InstancesDeleted => write!(f, "instances deleted"),
            ClusterStatus::DeletingDisks => write!(f, "deleting disks"),
            ClusterStatus::Deleted => write!(f, "deleted"),
            ClusterStatus::Resizing => write!(f, "resizing"),
            ClusterStatus::Stopping => write!(f, "stopping"),
            ClusterStatus::Starting => write!(f, "starting"),
            ClusterStatus::UpdatingConfiguration => write!(f, "updating configuration"),
            ClusterStatus::ComputeAvailable => write!(f, "compute available"),
            ClusterStatus::Installing => write!(f, "installing"),
            ClusterStatus::Deploying => write!(f, "deploying"),
            ClusterStatus::Updating => write!(f, "updating"),
            ClusterStatus::Deleting => write!(f, "deleting"),
        }
    }
}
impl std::cmp::PartialEq<&str> for ClusterStatus {
    fn eq(&self, other: &&str) -> bool {
        match self {
            ClusterStatus::Provisioning => *other == "provisioning",
            ClusterStatus::DisksAvailable => *other == "disks available",
            ClusterStatus::ExpandingDisks => *other == "expanding disks",
            ClusterStatus::Restarting => *other == "restarting",
            ClusterStatus::Available => *other == "available",
            ClusterStatus::Defunct => *other == "defunct",
            ClusterStatus::Inconsistent => *other == "inconsistent",
            ClusterStatus::Upgrading => *other == "upgrading",
            ClusterStatus::DeletingInstances => *other == "deleting instances",
            ClusterStatus::InstancesDeleted => *other == "instances deleted",
            ClusterStatus::DeletingDisks => *other == "deleting disks",
            ClusterStatus::Deleted => *other == "deleted",
            ClusterStatus::Resizing => *other == "resizing",
            ClusterStatus::Stopping => *other == "stopping",
            ClusterStatus::Starting => *other == "starting",
            ClusterStatus::UpdatingConfiguration => *other == "updating configuration",
            ClusterStatus::ComputeAvailable => *other == "compute available",
            ClusterStatus::Installing => *other == "installing",
            ClusterStatus::Deploying => *other == "deploying",
            ClusterStatus::Updating => *other == "updating",
            ClusterStatus::Deleting => *other == "deleting",
        }
    }
}
impl std::cmp::PartialEq<ClusterStatus> for &str {
    fn eq(&self, other: &ClusterStatus) -> bool {
        other == self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterUpgradeVersion {
    pub change: UpgradeChangeType,
    pub name: String,
    pub lts: bool,
    pub recommended: bool,
    pub tag: String,
    pub version: String,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAclRequest {
    pub cidr_blocks: Vec<AclCidrBlock>,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBackupRequest {
    pub description: String,
    pub source_cluster_id: ClusterId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBackupResponse {
    pub id: BackupId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClusterRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acl_id: Option<String>,
    pub description: String,
    pub disk_size_gb: i32,
    pub disk_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_iops: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_throughput: Option<i32>,
    pub instance_type: String,
    pub network_id: NetworkId,
    pub projection_level: ProjectionLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    pub server_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_backup_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_node_index: Option<i32>,
    pub topology: Topology,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_access: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_backup_project_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClusterResponse {
    pub id: ClusterId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSharedClusterDeploymentRequest {
    pub cluster: CreateSharedClusterRequest,
    pub acl: Acl,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSharedClusterRequest {
    pub name: String,
    pub provider: String,
    pub region: String,
    pub projection_level: ProjectionLevel,
    pub server_version: String,
    pub topology: Topology,
    pub deployment_tier: String,
    pub mutual_tls_enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSharedClusterResponse {
    pub id: ClusterId,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpandClusterDiskRequest {
    pub disk_size_gb: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_iops: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_throughput: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_type: Option<String>,
}

pub type Fields = HashMap<String, String>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBackupResponse {
    pub backup: Backup,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetClusterResponse {
    pub cluster: Cluster,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSharedClusterInitialCredentialsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credentials: Option<Credentials>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSharedClusterResponse {
    pub cluster: SharedCluster,
}

/// The health of the database
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Health {
    #[serde(rename = "degraded")]
    Degraded,
    #[serde(rename = "down")]
    Down,
    #[serde(rename = "health-reporting-error")]
    HealthReportingError,
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "issues-detected")]
    IssuesDetected,
}
impl std::fmt::Display for Health {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Health::Degraded => write!(f, "degraded"),
            Health::Down => write!(f, "down"),
            Health::HealthReportingError => write!(f, "health-reporting-error"),
            Health::Ok => write!(f, "ok"),
            Health::IssuesDetected => write!(f, "issues-detected"),
        }
    }
}
impl std::cmp::PartialEq<&str> for Health {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Health::Degraded => *other == "degraded",
            Health::Down => *other == "down",
            Health::HealthReportingError => *other == "health-reporting-error",
            Health::Ok => *other == "ok",
            Health::IssuesDetected => *other == "issues-detected",
        }
    }
}
impl std::cmp::PartialEq<Health> for &str {
    fn eq(&self, other: &Health) -> bool {
        other == self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListBackupsResponse {
    pub backups: Vec<Backup>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListClusterCreateVersionsResponse {
    pub versions: Vec<ClusterCreateVersion>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListClustersResponse {
    pub clusters: Vec<Cluster>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListClusterUpgradeVersionsResponse {
    pub versions: Vec<ClusterUpgradeVersion>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSharedClustersResponse {
    pub clusters: Vec<SharedCluster>,
}

/// The projection level of your database. Can be off, system or user
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProjectionLevel {
    Off,
    System,
    User,
}
impl std::fmt::Display for ProjectionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectionLevel::Off => write!(f, "off"),
            ProjectionLevel::System => write!(f, "system"),
            ProjectionLevel::User => write!(f, "user"),
        }
    }
}
impl std::cmp::PartialEq<&str> for ProjectionLevel {
    fn eq(&self, other: &&str) -> bool {
        match self {
            ProjectionLevel::Off => *other == "off",
            ProjectionLevel::System => *other == "system",
            ProjectionLevel::User => *other == "user",
        }
    }
}
impl std::cmp::PartialEq<ProjectionLevel> for &str {
    fn eq(&self, other: &ProjectionLevel) -> bool {
        other == self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResizeClusterRequest {
    pub target_size: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResizeClusterResponse {
    pub id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceIdentifier {
    pub id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestartClusterResponse {
    pub id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedCluster {
    pub created: DateTime<Utc>,
    pub name: String,
    pub id: ClusterId,
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
    pub acl_id: String,
    pub projection_level: ProjectionLevel,
    pub provider: String,
    pub region: String,
    pub server_version: String,
    pub server_version_tag: String,
    pub status: ClusterStatus,
    pub topology: Topology,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connection_url: Option<String>,
    pub mutual_tls_enabled: bool,
    pub per_node_cores: String,
    pub per_node_memory: i32,
    pub per_node_volume_size: i32,
    pub deployment_tier: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartClusterResponse {
    pub id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopClusterResponse {
    pub id: String,
}

/// Either single-node, three-node-multi-zone, three-node or five-node
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Topology {
    #[serde(rename = "single-node")]
    SingleNode,
    #[serde(rename = "three-node-multi-zone")]
    ThreeNodeMultiZone,
    #[serde(rename = "three-node")]
    ThreeNode,
    #[serde(rename = "five-node")]
    FiveNode,
}
impl std::fmt::Display for Topology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Topology::SingleNode => write!(f, "single-node"),
            Topology::ThreeNodeMultiZone => write!(f, "three-node-multi-zone"),
            Topology::ThreeNode => write!(f, "three-node"),
            Topology::FiveNode => write!(f, "five-node"),
        }
    }
}
impl std::cmp::PartialEq<&str> for Topology {
    fn eq(&self, other: &&str) -> bool {
        match self {
            Topology::SingleNode => *other == "single-node",
            Topology::ThreeNodeMultiZone => *other == "three-node-multi-zone",
            Topology::ThreeNode => *other == "three-node",
            Topology::FiveNode => *other == "five-node",
        }
    }
}
impl std::cmp::PartialEq<Topology> for &str {
    fn eq(&self, other: &Topology) -> bool {
        other == self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClusterRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acl_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protected: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSharedClusterRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mutual_tls_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acl_id: Option<String>,
}

/// The type of change in an upgrade
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UpgradeChangeType {
    Downgrade,
    Major,
    Patch,
}
impl std::fmt::Display for UpgradeChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpgradeChangeType::Downgrade => write!(f, "downgrade"),
            UpgradeChangeType::Major => write!(f, "major"),
            UpgradeChangeType::Patch => write!(f, "patch"),
        }
    }
}
impl std::cmp::PartialEq<&str> for UpgradeChangeType {
    fn eq(&self, other: &&str) -> bool {
        match self {
            UpgradeChangeType::Downgrade => *other == "downgrade",
            UpgradeChangeType::Major => *other == "major",
            UpgradeChangeType::Patch => *other == "patch",
        }
    }
}
impl std::cmp::PartialEq<UpgradeChangeType> for &str {
    fn eq(&self, other: &UpgradeChangeType) -> bool {
        other == self
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeClusterRequest {
    pub target_tag: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeClusterResponse {
    pub id: String,
}
