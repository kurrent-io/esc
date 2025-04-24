use chrono::{DateTime, Utc};

use super::common::{List, ToV1};
use super::infra::Provider;
use super::resources::OrgId;

#[derive(Deserialize, Serialize, Debug)]
pub struct ClusterAddresses {
    tcp: Vec<String>,
    grpc: String,
    ui: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cluster {
    pub id: esc_api::mesdb::ClusterId,
    #[serde(rename = "organizationId")]
    pub org_id: OrgId,
    pub project_id: esc_api::resources::ProjectId,
    pub network_id: esc_api::infra::NetworkId,
    pub description: String,
    pub provider: super::infra::Provider,
    pub region: String,
    pub topology: esc_api::mesdb::Topology,
    pub instance_type: String,
    pub disk_size_gb: usize,
    pub disk_type: String,
    pub server_version: String,
    pub projection_level: esc_api::mesdb::ProjectionLevel,
    pub status: String,
    pub created: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_iops: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disk_throughput: Option<i32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EnrichedCluster {
    #[serde(flatten)]
    cluster: Cluster,
    addresses: ClusterAddresses,
}

impl ToV1 for esc_api::mesdb::Cluster {
    type V1Type = EnrichedCluster;
    fn to_v1(self) -> Self::V1Type {
        let cluster = Cluster {
            created: self.created,
            description: self.description,
            disk_iops: self.disk_iops,
            disk_size_gb: self.disk_size_gb as usize,
            disk_throughput: self.disk_throughput,
            disk_type: self.disk_type,
            id: self.id,
            instance_type: self.instance_type,
            network_id: self.network_id,
            org_id: self.organization_id.to_v1(),
            project_id: self.project_id,
            projection_level: self.projection_level,
            provider: Provider::from_string(&self.provider),
            region: self.region,
            server_version: self.server_version,
            status: self.status.to_string(),
            topology: self.topology,
        };
        let mut tcp = Vec::new();
        let ui = format!("https://{}.mesdb.eventstore.cloud:2113", cluster.id.0);

        let grpc = if let esc_api::mesdb::Topology::ThreeNodeMultiZone = cluster.topology {
            for idx in 0..3 {
                tcp.push(format!(
                    "{}-{}.mesdb.eventstore.cloud:1113",
                    cluster.id.0, idx
                ));
            }

            format!(
                "esdb+discover://{}.mesdb.eventstore.cloud:2113",
                cluster.id.0
            )
        } else {
            tcp.push(format!("{}.mesdb.eventstore.cloud:1113", cluster.id.0));
            format!("esdb://{}.mesdb.eventstore.cloud:2113", cluster.id.0)
        };

        EnrichedCluster {
            cluster,
            addresses: ClusterAddresses { tcp, grpc, ui },
        }
    }
}

impl ToV1 for esc_api::mesdb::CreateClusterResponse {
    type V1Type = esc_api::mesdb::ClusterId;
    fn to_v1(self) -> Self::V1Type {
        self.id
    }
}

impl ToV1 for esc_api::mesdb::GetClusterResponse {
    type V1Type = EnrichedCluster;
    fn to_v1(self) -> Self::V1Type {
        self.cluster.to_v1()
    }
}

impl ToV1 for esc_api::mesdb::ListClustersResponse {
    type V1Type = List<EnrichedCluster>;
    fn to_v1(self) -> Self::V1Type {
        let l = self.clusters.into_iter().map(|c| c.to_v1()).collect();
        List(l)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Backup {
    pub id: esc_api::mesdb::BackupId,
    pub project_id: esc_api::resources::ProjectId,
    pub source_cluster_id: esc_api::mesdb::ClusterId,
    pub source_cluster_description: String,
    pub description: String,
    pub size_gb: usize,
    pub provider: Provider,
    pub region: String,
    pub status: String,
    pub created: String,
    pub linked_resource: Option<String>,
}

impl ToV1 for esc_api::mesdb::Backup {
    type V1Type = Backup;
    fn to_v1(self) -> Self::V1Type {
        Backup {
            created: self.created.to_rfc3339(),
            description: self.description,
            id: self.id,
            linked_resource: self.linked_resource,
            project_id: self.project_id,
            provider: Provider::from_string(&self.provider),
            region: self.region,
            size_gb: self.size_gb as usize,
            source_cluster_description: self.source_cluster_description,
            source_cluster_id: self.source_cluster_id,
            status: self.status.to_string(),
        }
    }
}

impl ToV1 for esc_api::mesdb::CreateBackupResponse {
    type V1Type = esc_api::mesdb::BackupId;
    fn to_v1(self) -> Self::V1Type {
        self.id
    }
}

impl ToV1 for esc_api::mesdb::GetBackupResponse {
    type V1Type = Backup;
    fn to_v1(self) -> Self::V1Type {
        self.backup.to_v1()
    }
}

impl ToV1 for esc_api::mesdb::ListBackupsResponse {
    type V1Type = List<Backup>;
    fn to_v1(self) -> Self::V1Type {
        let l = self.backups.into_iter().map(|b| b.to_v1()).collect();
        List(l)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharedCluster {
    pub id: esc_api::mesdb::ClusterId,
    pub org_id: OrgId,
    pub project_id: esc_api::resources::ProjectId,
    pub name: String,
    pub provider: Provider,
    pub region: String,
    pub topology: esc_api::mesdb::Topology,
    pub deployment_tier: String,
    pub mutual_tls_enabled: bool,
    pub per_node_cores: String,
    pub per_node_memory: usize,
    pub per_node_volume_size: usize,
    pub status: String,
    pub created: String,
    pub acl_id: String,
    pub projection_level: esc_api::mesdb::ProjectionLevel,
    pub server_version: String,
    pub server_version_tag: String,
}

impl ToV1 for esc_api::mesdb::SharedCluster {
    type V1Type = SharedCluster;
    fn to_v1(self) -> Self::V1Type {
        SharedCluster {
            created: self.created.to_rfc3339(),
            id: self.id,
            name: self.name,
            org_id: self.organization_id.to_v1(),
            project_id: self.project_id,
            provider: Provider::from_string(&self.provider),
            region: self.region,
            topology: self.topology,
            deployment_tier: self.deployment_tier,
            mutual_tls_enabled: self.mutual_tls_enabled,
            per_node_cores: self.per_node_cores,
            per_node_memory: self.per_node_memory as usize,
            per_node_volume_size: self.per_node_volume_size as usize,
            status: self.status.to_string(),
            acl_id: self.acl_id,
            projection_level: self.projection_level,
            server_version: self.server_version,
            server_version_tag: self.server_version_tag,
        }
    }
}

impl ToV1 for esc_api::mesdb::CreateSharedClusterResponse {
    type V1Type = esc_api::mesdb::ClusterId;
    fn to_v1(self) -> Self::V1Type {
        self.id
    }
}

impl ToV1 for esc_api::mesdb::GetSharedClusterResponse {
    type V1Type = SharedCluster;
    fn to_v1(self) -> Self::V1Type {
        self.cluster.to_v1()
    }
}

impl ToV1 for esc_api::mesdb::ListSharedClustersResponse {
    type V1Type = List<SharedCluster>;
    fn to_v1(self) -> Self::V1Type {
        let l = self.clusters.into_iter().map(|c| c.to_v1()).collect();
        List(l)
    }
}
