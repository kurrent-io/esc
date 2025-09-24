#![allow(clippy::single_match)]

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

mod config;
mod constants;
mod output;
mod utils;
mod v1;

use cidr::Cidr;
use esc_api::resources::MfaStatus;
use esc_client_base::build_http_client;

use esc_api::{GroupId, MemberId, OrgId};
use output::OutputFormat;
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "esc",
    about = "EventStoreDB Cloud tool.",
    author = "Event Store Limited <ops@eventstore.com>"
)]
pub struct Opt {
    #[structopt(
        long,
        help = "Prints a verbose output during the program execution",
        global = true
    )]
    debug: bool,

    #[structopt(
        long = "json",
        help = "Renders the classic ESC output as JSON (some differences from API)",
        global = true
    )]
    render_in_json: bool,

    #[structopt(long = "fmt", parse(try_from_str = parse_output_format), default_value = "", help = "Selects the output format", global = true)]
    output_format: OutputFormat,

    #[structopt(
        long,
        help = "Refresh token, useful if you intend to use esc in a CI/scripting setting for example",
        global = true
    )]
    refresh_token: Option<String>,

    #[structopt(
        long,
        help = "If true never prompt for authentication details",
        global = true
    )]
    noninteractive: bool,

    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    Access(Access),
    Audit(Audit),
    Resources(Resources),
    Infra(Infra),
    Integrations(Integrations),
    Profiles(Profiles),
    Mesdb(Mesdb),
    Orchestrate(Orchestrate),
    #[structopt(about = "Prints Bash completion script in STDOUT")]
    GenerateBashCompletion,
    #[structopt(about = "Prints Zsh completion script in STDOUT")]
    GenerateZshCompletion,
    #[structopt(about = "Prints Powershell completion script in STDOUT")]
    GeneratePowershellCompletion,
}

#[derive(StructOpt, Debug)]
#[structopt(
    about = "Gathers tokens, groups, members, invites, policies and settings management commands"
)]
struct Access {
    #[structopt(subcommand)]
    access_command: AccessCommand,
}

#[derive(StructOpt, Debug)]
enum AccessCommand {
    Tokens(Tokens),
    Groups(Groups),
    Invites(Invites),
    Policies(Policies),
    Members(Members),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Gathers audit logs (by organization and user) management commands")]
struct Audit {
    #[structopt(subcommand)]
    audit_command: AuditCommand,
}

#[derive(Debug, StructOpt)]
enum AuditCommand {
    Organization(AuditOrganization),
    User(AuditUser),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Gathers organizations audit commands")]
struct AuditOrganization {
    #[structopt(subcommand)]
    organization_command: AuditOrganizationCommand,
}

#[derive(Debug, StructOpt)]
enum AuditOrganizationCommand {
    Get(GetOrganizationAudit),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "get an organization's audit logs")]
struct GetOrganizationAudit {
    #[structopt(short, long, parse(try_from_str = parse_org_id), default_value = "", help = "The id of the organization for which to get audit logs")]
    org_id: esc_api::resources::OrganizationId,
    #[structopt(short, long, help = "The timestamp until when to retrieve audit logs")]
    before: Option<String>,
    #[structopt(
        short,
        long,
        help = "The timestamp as from when to retrieve audit logs"
    )]
    after: Option<String>,
    #[structopt(short, long, help = "The maximum number of records to retrieve")]
    limit: Option<String>,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Gathers user audit commands")]
struct AuditUser {
    #[structopt(subcommand)]
    user_command: AuditUserCommand,
}

#[derive(Debug, StructOpt)]
enum AuditUserCommand {
    Get(GetUserAudit),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "get a user's audit logs")]
struct GetUserAudit {
    #[structopt(short, long, help = "The timestamp until when to retrieve audit logs")]
    before: Option<String>,
    #[structopt(
        short,
        long,
        help = "The timestamp as from when to retrieve audit logs"
    )]
    after: Option<String>,
    #[structopt(short, long, help = "The maximum number of records to retrieve")]
    limit: Option<String>,
    #[structopt(
        short,
        long,
        help = "The id of the organization for which to retrieve audit logs"
    )]
    org_id: Option<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers tokens management commands")]
struct Tokens {
    #[structopt(subcommand)]
    tokens_command: TokensCommand,
}

#[derive(StructOpt, Debug)]
enum TokensCommand {
    Create(CreateToken),
    Display(Display),
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Create an access token")]
struct CreateToken {
    #[structopt(long, short, parse(try_from_str = parse_email), help = "The email you used to create an EventStoreDB Cloud")]
    email: Option<String>,

    #[structopt(
        long,
        help = "Set this parameter if you don't want to give your password safely (non-interactive)"
    )]
    unsafe_password: Option<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Display your current refresh token")]
struct Display {}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers groups management commands")]
struct Groups {
    #[structopt(subcommand)]
    groups_command: GroupsCommand,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers members management commands")]
struct Members {
    #[structopt(subcommand)]
    members_command: MembersCommand,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers invites management commands")]
struct Invites {
    #[structopt(subcommand)]
    invites_command: InvitesCommand,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers integration management commands")]
struct Integrations {
    #[structopt(subcommand)]
    integration_command: IntegrationsCommand,
}

#[derive(StructOpt, Debug)]
struct Policies {
    #[structopt(subcommand)]
    policies_command: PoliciesCommand,
}

#[derive(StructOpt, Debug)]
enum PoliciesCommand {
    Create(CreatePolicy),
    Update(UpdatePolicy),
    Get(GetPolicy),
    Delete(DeletePolicy),
    List(ListPolicies),
}

#[derive(StructOpt, Debug)]
struct CreatePolicy {
    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the policy will relate to")]
    org_id: OrgId,
    #[structopt(long, short, help = "Policy's name")]
    name: String,
    #[structopt(long, short, help = "Policy's subjects")]
    subjects: Vec<String>,
    #[structopt(long, short, help = "Policy's resources")]
    resources: Vec<String>,
    #[structopt(long, short, help = "Policy's actions")]
    actions: Vec<String>,
    #[structopt(long, short, help = "Policy's effect")]
    effect: String,
}

#[derive(StructOpt, Debug)]
struct UpdatePolicy {
    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the policy is related to")]
    org_id: OrgId,
    #[structopt(long, short, parse(try_from_str = parse_policy_id), help = "Policy's id")]
    policy: esc_api::PolicyId,
    #[structopt(long, short, help = "Policy's name")]
    name: String,
    #[structopt(long, short, help = "Policy's subjects")]
    subjects: Vec<String>,
    #[structopt(long, short, help = "Policy's resources")]
    resources: Vec<String>,
    #[structopt(long, short, help = "Policy's actions")]
    actions: Vec<String>,
    #[structopt(long, short, help = "Policy's effect")]
    effect: String,
}

#[derive(StructOpt, Debug)]
struct GetPolicy {
    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the policy is related to")]
    org_id: OrgId,

    #[structopt(long, short, parse(try_from_str = parse_policy_id), help = "Policy's id")]
    policy: esc_api::PolicyId,
}

#[derive(StructOpt, Debug)]
struct DeletePolicy {
    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the policy is related to")]
    org_id: OrgId,

    #[structopt(long, short, parse(try_from_str = parse_policy_id), help = "Policy's id")]
    policy: esc_api::PolicyId,
}

#[derive(StructOpt, Debug)]
struct ListPolicies {
    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the policy is related to")]
    org_id: OrgId,
}

#[derive(StructOpt, Debug)]
enum GroupsCommand {
    Create(CreateGroup),
    Update(UpdateGroup),
    Get(GetGroup),
    Delete(DeleteGroup),
    List(ListGroups),
}

#[derive(StructOpt, Debug)]
enum MembersCommand {
    Get(GetMember),
    List(ListMembers),
    Update(UpdateMember),
    Delete(DeleteMember),
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Read information about a Member")]
struct GetMember {
    #[structopt(long, short, parse(try_from_str = parse_member_id))]
    id: MemberId,

    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "")]
    org_id: OrgId,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "List members")]
struct ListMembers {
    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the members relate to")]
    org_id: OrgId,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Update a member")]
struct UpdateMember {
    #[structopt(long, short, parse(try_from_str = parse_member_id), help = "The member id")]
    id: MemberId,

    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the member will relate to")]
    org_id: OrgId,

    #[structopt(
        long,
        short,
        parse(try_from_str),
        help = "Specifies whether the member is active."
    )]
    active: bool,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Deletes a Member")]
struct DeleteMember {
    #[structopt(long, short, parse(try_from_str = parse_member_id))]
    id: MemberId,

    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "")]
    org_id: OrgId,
}

#[derive(StructOpt, Debug)]
enum UserCommand {
    List,
    Revoke,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Create a group")]
struct CreateGroup {
    #[structopt(long, short, help = "The group's name")]
    name: String,

    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the group will relate to")]
    org_id: OrgId,

    #[structopt(long, short, help = "The members of the group")]
    members: Vec<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Update a group")]
struct UpdateGroup {
    #[structopt(long, short, parse(try_from_str = parse_group_id), help = "The group's id")]
    id: GroupId,

    #[structopt(long, short, help = "The group's name")]
    name: Option<String>,

    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the group will relate to")]
    org_id: OrgId,

    #[structopt(long, short, help = "The members of the group")]
    members: Option<Vec<String>>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Read a group information")]
struct GetGroup {
    #[structopt(long, short, parse(try_from_str = parse_group_id))]
    id: GroupId,

    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "")]
    org_id: OrgId,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Delete a group")]
struct DeleteGroup {
    #[structopt(long, short, parse(try_from_str = parse_group_id), help = "The group's id")]
    id: GroupId,

    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the group will relate to")]
    org_id: OrgId,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "List groups")]
struct ListGroups {
    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the groups relate to")]
    org_id: OrgId,
}

#[derive(StructOpt, Debug)]
enum InvitesCommand {
    Create(CreateInvite),
    Resend(ResendInvite),
    Delete(DeleteInvite),
    List(ListInvites),
}

#[derive(StructOpt, Debug)]
struct CreateInvite {
    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the invite will relate to")]
    org_id: OrgId,

    #[structopt(long, short, parse(try_from_str = parse_email), help = "The email that will receive the invite")]
    email: String,

    #[structopt(long, short, parse(try_from_str = parse_group_id), help = "Group(s) the invite will associate the member with after confirmation")]
    group: Option<Vec<GroupId>>,
}

#[derive(StructOpt, Debug)]
struct ResendInvite {
    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the invite will relate to")]
    org_id: OrgId,

    #[structopt(long, short, parse(try_from_str = parse_invite_id), help = "The invite's id")]
    id: esc_api::access::InviteId,
}

#[derive(StructOpt, Debug)]
struct DeleteInvite {
    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the invite relates to")]
    org_id: OrgId,

    #[structopt(long, short, parse(try_from_str = parse_invite_id), help = "The invite's id")]
    id: esc_api::access::InviteId,
}

#[derive(StructOpt, Debug)]
struct ListInvites {
    #[structopt(long, short, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the invites relate to")]
    org_id: esc_api::resources::OrganizationId,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers networks and peering management commands")]
struct Infra {
    #[structopt(subcommand)]
    infra_command: InfraCommand,
}

#[derive(StructOpt, Debug)]
enum InfraCommand {
    Acls(Acls),
    Networks(Networks),
    Peerings(Peerings),
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers acls management commands")]
struct Acls {
    #[structopt(subcommand)]
    acls_command: AclsCommand,
}

#[derive(StructOpt, Debug)]
enum AclsCommand {
    Create(CreateAcl),
    Delete(DeleteAcl),
    Get(GetAcl),
    List(ListAcls),
    Update(UpdateAcl),
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Create an acl")]
struct CreateAcl {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the acl will relate to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the acl will relate to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, parse(try_from_str = parse_cidr_input), help = "The CIDR blocks who will have access. Format: \"<cidr>,<optional_comment>\"")]
    cidr_blocks: Vec<esc_api::infra::AclCidrBlock>,

    #[structopt(long, help = "Human-readable description of the acl")]
    description: String,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Deletes an acl")]
struct DeleteAcl {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the acl relates to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the acl relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_acl_id), help = "A acl's id")]
    id: esc_api::infra::AclId,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Reads an acl information")]
struct GetAcl {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the acl relates to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the acl relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_acl_id), help = "An acl's id")]
    id: esc_api::infra::AclId,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "List acls of an organization, given a project")]
struct ListAcls {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the acls relate to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the acls relate to")]
    project_id: esc_api::resources::ProjectId,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Updates an acl")]
struct UpdateAcl {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the acl relates to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the acl relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_acl_id), help = "An acl's id")]
    id: esc_api::infra::AclId,

    #[structopt(long, parse(try_from_str = parse_cidr_input), help = "The CIDR blocks who will have access. Format: \"<cidr>,<optional_comment>\"")]
    cidr_blocks: Vec<esc_api::infra::AclCidrBlock>,

    #[structopt(long, help = "A human-readable acl's description")]
    description: Option<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers networks management commands")]
struct Networks {
    #[structopt(subcommand)]
    networks_command: NetworksCommand,
}

#[derive(StructOpt, Debug)]
enum NetworksCommand {
    Create(CreateNetwork),
    Delete(DeleteNetwork),
    Get(GetNetwork),
    List(ListNetworks),
    Update(UpdateNetwork),
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Create a network")]
struct CreateNetwork {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the network will relate to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the network will relate to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, parse(try_from_str = parse_provider), help = "The cloud provider: aws, gcp or azure")]
    provider: esc_api::infra::Provider,

    #[structopt(long, parse(try_from_str = parse_cidr), help = "Classless Inter-Domain Routing block (CIDR)")]
    cidr_block: Option<cidr::Ipv4Cidr>,

    #[structopt(long, help = "Human-readable description of the network")]
    description: String,

    #[structopt(long, help = "Cloud provider region")]
    region: String,

    #[structopt(
        long,
        help = "Networks with public access enabled can have clusters with public access enabled, whereas networks without can only be accessed via peering. Defaults to false."
    )]
    public_access: bool,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Delete a network")]
struct DeleteNetwork {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the network relates to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the network relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_network_id), help = "A network's id")]
    id: esc_api::infra::NetworkId,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Read a network information")]
struct GetNetwork {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the network relates to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the network relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_network_id), help = "A network's id")]
    id: esc_api::infra::NetworkId,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "List networks of an organization, given a project")]
struct ListNetworks {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the networks relate to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the networks relate to")]
    project_id: esc_api::resources::ProjectId,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Update network")]
struct UpdateNetwork {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the network relates to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the network relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_network_id), help = "A network's id")]
    id: esc_api::infra::NetworkId,

    #[structopt(long, help = "A human-readable network's description")]
    description: String,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers peering management commands")]
struct Peerings {
    #[structopt(subcommand)]
    peerings_command: PeeringsCommand,
}

#[derive(StructOpt, Debug)]
enum PeeringsCommand {
    Create(CreatePeering),
    Delete(DeletePeering),
    Get(GetPeering),
    List(ListPeerings),
    Update(UpdatePeering),
}

#[derive(StructOpt, Clone, Debug)]
#[structopt(about = "Create a peering")]
struct CreatePeering {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the peering will relate to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the peering will relate to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, parse(try_from_str = parse_network_id), default_value = "", help = "The network id the peering will relate to")]
    network_id: esc_api::infra::NetworkId,

    #[structopt(long, help = "Your cloud provider account id")]
    peer_account_id: String,

    #[structopt(long, help = "Your cloud provider network id")]
    peer_network_id: String,

    #[structopt(long, help = "Human-readable description for your peering")]
    description: String,

    #[structopt(long, help = "Your cloud provider network region")]
    peer_network_region: String,

    #[structopt(long, help = "Your network routes")]
    routes: Vec<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Delete a peering")]
struct DeletePeering {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the peering relates to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the peering relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_peering_id), help = "The peering's id")]
    id: esc_api::PeeringId,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Read a peering information")]
struct GetPeering {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the peering relates to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the peering relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_peering_id), help = "The peering's id")]
    id: esc_api::PeeringId,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "List all peering related an organization, given a project id")]
struct ListPeerings {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the peerings relate to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the peerings relate to")]
    project_id: esc_api::resources::ProjectId,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Update a peering")]
struct UpdatePeering {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the peering relates to")]
    org_id: esc_api::resources::OrganizationId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the peering relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_peering_id), help = "The peering's id")]
    id: esc_api::PeeringId,

    #[structopt(long, help = "A human-readable description for your peering")]
    description: String,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers ESC local profile management commands")]
struct Profiles {
    #[structopt(subcommand)]
    profiles_command: ProfilesCommand,
}

#[derive(StructOpt, Debug)]
enum ProfilesCommand {
    Set(ProfileProp),
    Get(OptionalNamedProp),
    Delete(NamedProp),
    List,
    Default(ProfileDefault),
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Set a local profile parameter value")]
struct ProfileProp {
    #[structopt(long, short, help = "The profile's name")]
    profile: String,

    #[structopt(long, short, parse(try_from_str = parse_context_prop_name), help = "Name of the parameter")]
    name: ProfilePropName,

    #[structopt(long, short, help = "Parameter's value")]
    value: String,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Delete a profile parameter")]
struct NamedProp {
    #[structopt(long, short, help = "Profile's name")]
    profile: String,

    #[structopt(long, short, parse(try_from_str = parse_context_prop_name), help = "Name of the parameter")]
    name: ProfilePropName,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Read a profile parameter(s)")]
struct OptionalNamedProp {
    #[structopt(long, short, help = "Profile's name")]
    profile: String,

    #[structopt(long, short, parse(try_from_str = parse_context_prop_name), help = "Name of the parameter. If not mentioned, list all the profile's parameters")]
    name: Option<ProfilePropName>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers default profile commands")]
struct ProfileDefault {
    #[structopt(subcommand)]
    default_command: ProfileDefaultCommand,
}

#[derive(StructOpt, Debug)]
enum ProfileDefaultCommand {
    Get(GetProfileDefault),
    Set(ProfileDefaultSet),
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Read the current default local profile name")]
struct GetProfileDefault {}

#[derive(StructOpt, Debug)]
#[structopt(about = "Set default local profile")]
struct ProfileDefaultSet {
    #[structopt(long, short, help = "Profile's name")]
    value: String,
}

#[derive(Debug, Copy, Clone)]
enum ProfilePropName {
    OrgId,
    ProjectId,
    ApiBaseUrl,
    Fmt,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Gathers organizations and projects management commands")]
struct Resources {
    #[structopt(subcommand)]
    resources_command: ResourcesCommand,
}

#[derive(Debug, StructOpt)]
enum ResourcesCommand {
    Organizations(Organizations),
    Projects(Projects),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Gathers organizations management commands")]
struct Organizations {
    #[structopt(subcommand)]
    organizations_command: OrganizationsCommand,
}

#[derive(Debug, StructOpt)]
enum OrganizationsCommand {
    Create(CreateOrganization),
    Update(UpdateOrganization),
    Get(GetOrganization),
    Delete(DeleteOrganization),
    List(ListOrganizations),
    GetMfaStatus(GetOrganizationMfaStatus),
    UpdateMfaStatus(UpdateOrganizationMfaStatus),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Create an organization")]
struct CreateOrganization {
    #[structopt(long, short)]
    name: String,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Update an organization")]
struct UpdateOrganization {
    #[structopt(short, long, parse(try_from_str = parse_org_id), help = "The id of the organization you want to update")]
    id: OrgId,

    #[structopt(long, short)]
    name: String,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "read an organization's information")]
struct GetOrganization {
    #[structopt(short, long, parse(try_from_str = parse_org_id), default_value = "", help = "The id of the organization you want to read information from")]
    id: OrgId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "read an organization's MFA status")]
struct GetOrganizationMfaStatus {
    #[structopt(short, long, parse(try_from_str = parse_org_id), default_value = "", help = "The id of the organization you want to read MFA status of")]
    id: OrgId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "updates an organization's MFA status")]
struct UpdateOrganizationMfaStatus {
    #[structopt(short, long, parse(try_from_str = parse_org_id), default_value = "", help = "The id of the organization you want to update MFA status for")]
    id: OrgId,

    #[structopt(
        long,
        short,
        parse(try_from_str),
        help = "Specifies whether MFA is requied for the org."
    )]
    enabled: bool,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Delete an organization")]
struct DeleteOrganization {
    #[structopt(short, long, parse(try_from_str = parse_org_id), help = "The id of the organization you want to delete")]
    id: OrgId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "List organizations")]
struct ListOrganizations {}

#[derive(Debug, StructOpt)]
#[structopt(about = "Gathers projects management commands")]
struct Projects {
    #[structopt(subcommand)]
    projects_command: ProjectsCommand,
}

#[derive(Debug, StructOpt)]
enum ProjectsCommand {
    Create(CreateProject),
    Update(UpdateProject),
    Get(GetProject),
    Delete(DeleteProject),
    List(ListProjects),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Create a project")]
struct CreateProject {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the project will relate to")]
    org_id: OrgId,

    #[structopt(long, short, help = "Project's name")]
    name: String,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Update a project")]
struct UpdateProject {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the project is related to")]
    org_id: OrgId,

    #[structopt(long, short, parse(try_from_str = parse_project_id), default_value = "", help = "The id of the project you want to update")]
    id: esc_api::resources::ProjectId,

    #[structopt(long, short, help = "New project's name")]
    name: String,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Get a project information")]
struct GetProject {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the project is related to")]
    org_id: OrgId,

    #[structopt(long, short, parse(try_from_str = parse_project_id), default_value = "", help = "The id of the project you want to read information from")]
    id: esc_api::resources::ProjectId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Delete a project")]
struct DeleteProject {
    #[structopt(long, parse(try_from_str = parse_org_id), help = "The organization id the project is related to")]
    org_id: OrgId,

    #[structopt(long, short, parse(try_from_str = parse_project_id), help = "The id of the project you want to delete")]
    id: esc_api::resources::ProjectId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "List an organization's projects")]
struct ListProjects {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "Organization's id")]
    org_id: OrgId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Gathers cluster management commands")]
struct Mesdb {
    #[structopt(subcommand)]
    mesdb_command: MesdbCommand,
}

#[derive(Debug, StructOpt)]
enum MesdbCommand {
    Clusters(Clusters),
    Backups(Backups),
    SharedClusters(SharedClusters),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Gathers cluster management commands")]
struct Clusters {
    #[structopt(subcommand)]
    clusters_command: ClustersCommand,
}

#[derive(Debug, StructOpt)]
enum ClustersCommand {
    Create(CreateCluster),
    Get(GetCluster),
    List(ListClusters),
    Update(UpdateCluster),
    Delete(DeleteCluster),
    Expand(ExpandCluster),
    Stop(StopCluster),
    Start(StartCluster),
    Resize(ResizeCluster),
    Upgrade(UpgradeCluster),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Create a cluster")]
struct CreateCluster {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster will relate to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster will relate to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(
        long,
        help = "The optional id of the project in which the source backup is located. This parameter is required for cross-project backup restores"
    )]
    source_project_id: Option<String>,

    #[structopt(long, help = "The ID of an ACL if one is being used")]
    acl_id: Option<String>,

    #[structopt(long, parse(try_from_str = parse_network_id), help = "The network id the cluster will be set on")]
    network_id: esc_api::infra::NetworkId,

    #[structopt(long, help = "A human-readable description of the cluster")]
    description: String,

    #[structopt(long, parse(try_from_str = parse_topology), help = "Either single-node or three-node-multi-zone")]
    topology: esc_api::mesdb::Topology,

    #[structopt(
        long,
        help = "Type of instance, based on its hardware. For example, it could be F1 for a micro or C4 for a large instance"
    )]
    instance_type: String,

    #[structopt(long, help = "Total disk capacity in Gigabytes (GB)")]
    disk_size_in_gb: i32,

    #[structopt(
        long,
        help = "Type of disk. For example, if you are using AWS as a provider, it could be GP2"
    )]
    disk_type: String,

    #[structopt(long, help = "EventStoreDB server version")]
    server_version: String,

    #[structopt(
        long,
        parse(try_from_str = parse_projection_level),
        help = "The projection level of your database. Can be off, system or user "
    )]
    projection_level: esc_api::mesdb::ProjectionLevel,

    #[structopt(long, help = "Optional id of backup to restore")]
    source_backup_id: Option<String>,

    #[structopt(long, help = "Optional IOPS number for disk (only AWS)")]
    pub disk_iops: Option<i32>,

    #[structopt(long, help = "Throughput in Mb/s for disk (only AWS)")]
    pub disk_throughput: Option<i32>,

    #[structopt(long, help = "If set, this cluster will be publicly accessible")]
    pub public_access: Option<bool>,

    #[structopt(long, help = "The protected flag prevents from accidental deletion")]
    protected: Option<bool>,

    #[structopt(long, help = "The provider of the cluster")]
    provider: Option<String>,

    #[structopt(long, help = "The region of the cluster")]
    region: Option<String>,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Get a cluster information")]
struct GetCluster {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_cluster_id), help = "Cluster's id")]
    id: esc_api::ClusterId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "List all clusters of an organization, given a project id")]
struct ListClusters {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "An organization's id")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "An project id that belongs to an organization pointed by --org-id")]
    project_id: esc_api::resources::ProjectId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Update a cluster")]
struct UpdateCluster {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_cluster_id), help = "Id of the cluster you want to update")]
    id: esc_api::ClusterId,

    #[structopt(long, help = "The ACL id used by a cluster")]
    acl_id: Option<String>,

    #[structopt(long, help = "A human-readable description of the cluster")]
    description: Option<String>,

    #[structopt(long, help = "The protected flag prevents from accidental deletion")]
    protected: Option<bool>,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Delete a cluster")]
struct DeleteCluster {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_cluster_id), help = "Id of the cluster you want to delete")]
    id: esc_api::ClusterId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Expand a cluster")]
struct ExpandCluster {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_cluster_id), help = "Id of the cluster you want to expand")]
    id: esc_api::ClusterId,

    #[structopt(long, help = "Disk size in GB")]
    disk_size_in_gb: i32,

    #[structopt(long, help = "IOPS number for disk (only AWS)")]
    disk_iops: Option<i32>,

    #[structopt(long, help = "Throughput in Mb/s for disk (only AWS)")]
    disk_throughput: Option<i32>,

    #[structopt(long, help = "Optional disk type")]
    disk_type: Option<String>,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Resize a cluster")]
struct ResizeCluster {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_cluster_id), help = "Id of the cluster you want to resize")]
    id: esc_api::ClusterId,

    #[structopt(long, help = "The target instance size. (C4, M8, etc)")]
    target_size: String,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Stops a cluster")]
struct StopCluster {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_cluster_id), help = "Id of the cluster you want to resize")]
    id: esc_api::ClusterId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Starts a cluster")]
struct StartCluster {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_cluster_id), help = "Id of the cluster you want to resize")]
    id: esc_api::ClusterId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Upgrade a cluster")]
struct UpgradeCluster {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_cluster_id), help = "Id of the cluster you want to upgrade")]
    id: esc_api::ClusterId,

    #[structopt(
        long,
        help = "The target tag you want to upgrade to. This must include the full version (23.10.1)."
    )]
    target_tag: String,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Gathers shared cluster management commands")]
struct SharedClusters {
    #[structopt(subcommand)]
    shared_clusters_command: SharedClustersCommand,
}

#[derive(Debug, StructOpt)]
enum SharedClustersCommand {
    Create(CreateSharedCluster),
    List(ListSharedClusters),
    Delete(DeleteSharedCluster),
    Get(GetSharedCluster),
}

#[derive(Debug, StructOpt)]
struct CreateSharedCluster {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, parse(try_from_str = parse_acl_id), help = "The acl id the cluster relates to")]
    acl_id: esc_api::infra::AclId,

    #[structopt(long, help = "The name of the cluster")]
    name: String,

    #[structopt(long, help = "The provider of the cluster")]
    provider: String,

    #[structopt(long, help = "The region of the cluster")]
    region: String,

    #[structopt(long, help = "The deployment tier of the cluster")]
    deployment_tier: String,

    #[structopt(long, help = "Whether mutual TLS is enabled for the cluster")]
    mutual_tls_enabled: bool,

    #[structopt(
        long,
        parse(try_from_str = parse_projection_level),
        help = "The projection level of your database. Can be off, system or user "
    )]
    projection_level: esc_api::mesdb::ProjectionLevel,

    #[structopt(long, help = "The server version of the cluster")]
    server_version: String,

    #[structopt(long, parse(try_from_str = parse_topology), help = "Either single-node or three-node-multi-zone")]
    topology: esc_api::mesdb::Topology,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "List all shared clusters of an organization, given a project id")]
struct ListSharedClusters {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster relates to")]
    project_id: esc_api::resources::ProjectId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Delete a shared cluster")]
struct DeleteSharedCluster {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_cluster_id), help = "Id of the cluster you want to delete")]
    id: esc_api::ClusterId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Get a shared cluster")]
struct GetSharedCluster {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_cluster_id), help = "Id of the cluster you want to get")]
    id: esc_api::ClusterId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Gathers backup management commands")]
struct Backups {
    #[structopt(subcommand)]
    backups_command: BackupsCommand,
}

#[derive(Debug, StructOpt)]
enum BackupsCommand {
    Create(CreateBackup),
    Get(GetBackup),
    List(ListBackups),
    Delete(DeleteBackup),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Create a backup")]
struct CreateBackup {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the backup will relate to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the backup will relate to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, parse(try_from_str = parse_cluster_id), help = "The id of the cluster to create backup of")]
    source_cluster_id: esc_api::ClusterId,

    #[structopt(long, help = "A human-readable description of the backup")]
    description: String,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Get information about a single backup")]
struct GetBackup {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the backup relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the backup relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_backup_id), help = "Backup's id")]
    id: esc_api::BackupId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "List all backups of an organization, given a project id")]
struct ListBackups {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "An organization's id")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "An project id that belongs to an organization pointed by --org-id")]
    project_id: esc_api::resources::ProjectId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Delete a backup")]
struct DeleteBackup {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the backup relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the backup relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_backup_id), help = "Id of the backup you want to delete")]
    id: esc_api::BackupId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Gathers jobs management commands")]
struct Orchestrate {
    #[structopt(subcommand)]
    orchestrate_command: OrchestrateCommand,
}

#[derive(Debug, StructOpt)]
enum OrchestrateCommand {
    Jobs(Jobs),
    History(History),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Gathers jobs management commands")]
struct Jobs {
    #[structopt(subcommand)]
    jobs_command: JobsCommand,
}

#[derive(Debug, StructOpt)]
enum JobsCommand {
    Create(CreateJob),
    Get(GetJob),
    List(ListJobs),
    Delete(DeleteJob),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Create a job")]
struct CreateJob {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the job will relate to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the job will relate to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, help = "A human-readable description of the job")]
    description: String,

    #[structopt(long, help = "The schedule in semi-cron format")]
    schedule: String,

    #[structopt(subcommand)]
    job_type: CreateJobType,
}

#[derive(Debug, StructOpt)]
enum CreateJobType {
    ScheduledBackup(ScheduledBackupArgs),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Scheduled backup args")]
struct ScheduledBackupArgs {
    #[structopt(long, short, help = "Description to give each backup")]
    description: String,
    #[structopt(long, short, help = "Max number of backups to keep")]
    max_backup_count: i32,
    #[structopt(long, short, parse(try_from_str = parse_cluster_id), help = "Id of the cluster you want to backup")]
    cluster_id: esc_api::ClusterId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Get job information")]
struct GetJob {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_job_id), help = "Job's id")]
    id: esc_api::JobId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "List all jobs of an organization, given a project id")]
struct ListJobs {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "An organization's id")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "An project id that belongs to an organization pointed by --org-id")]
    project_id: esc_api::resources::ProjectId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Delete a job")]
struct DeleteJob {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "The organization id the cluster relates to")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "The project id the cluster relates to")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, short, parse(try_from_str = parse_job_id), help = "Id of the job you want to delete")]
    id: esc_api::JobId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Gathers jobs management commands")]
struct History {
    #[structopt(subcommand)]
    history_command: HistoryCommand,
}

#[derive(Debug, StructOpt)]
enum HistoryCommand {
    List(ListHistory),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Show job history")]
struct ListHistory {
    #[structopt(long, parse(try_from_str = parse_org_id), default_value = "", help = "An organization's id")]
    org_id: OrgId,

    #[structopt(long, parse(try_from_str = parse_project_id), default_value = "", help = "An project id that belongs to an organization pointed by --org-id")]
    project_id: esc_api::resources::ProjectId,

    #[structopt(long, parse(try_from_str = parse_job_id), help = "A job ID")]
    job_id: Option<esc_api::JobId>,
}

#[derive(StructOpt, Debug)]
enum IntegrationsCommand {
    List(ListIntegrations),
    Create(CreateIntegration),
    Delete(DeleteIntegration),
    Get(GetIntegration),
    Update(UpdateIntegration),
    TestIntegration(TestIntegration),
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Creates a new integration")]
pub struct CreateIntegration {
    #[structopt(long, help="The id of the organization",  parse(try_from_str = parse_org_id), default_value = "")]
    pub organization_id: OrgId,
    #[structopt(long, help="The id of the project",  parse(try_from_str = parse_project_id), default_value = "")]
    pub project_id: esc_api::resources::ProjectId,
    #[structopt(subcommand)]
    pub data: CreateIntegrationData,
    #[structopt(long)]
    pub description: String,
}

#[derive(Debug, StructOpt)]
pub enum CreateIntegrationData {
    OpsGenie(CreateOpsGenieIntegrationData),
    Slack(CreateSlackIntegrationData),
}

#[derive(Debug, StructOpt)]
#[structopt()]
pub struct CreateOpsGenieIntegrationData {
    #[structopt(long, help = "API key used with the Ops Genie integration API")]
    pub api_key: String,
}

#[derive(Debug, StructOpt)]
#[structopt()]
pub struct CreateSlackIntegrationData {
    #[structopt(long, help = "Slack Channel to send messages to")]
    pub channel_id: String,
    #[structopt(long, help = "Integration source")]
    pub source: Option<String>,
    #[structopt(long, help = "API token for the Slack bot")]
    pub token: String,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "deletes a integration")]
pub struct DeleteIntegration {
    #[structopt(long, help="The id of the organization",  parse(try_from_str = parse_org_id), default_value = "")]
    pub organization_id: OrgId,
    #[structopt(long, help="The id of the project",  parse(try_from_str = parse_project_id), default_value = "")]
    pub project_id: esc_api::resources::ProjectId,
    #[structopt(long, help = "The id of the integration")]
    pub integration_id: String,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "retrieves a integration")]
pub struct GetIntegration {
    #[structopt(long, help="The id of the organization",  parse(try_from_str = parse_org_id), default_value = "")]
    pub organization_id: OrgId,
    #[structopt(long, help="The id of the project",  parse(try_from_str = parse_project_id), default_value = "")]
    pub project_id: esc_api::resources::ProjectId,
    #[structopt(long, help = "The id of the integration")]
    pub integration_id: String,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "list all integrations")]
pub struct ListIntegrations {
    #[structopt(long, help="The id of the organization",  parse(try_from_str = parse_org_id), default_value = "")]
    pub organization_id: OrgId,
    #[structopt(long, help="The id of the project",  parse(try_from_str = parse_project_id), default_value = "")]
    pub project_id: esc_api::resources::ProjectId,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "Sends a message to an integration sink")]
pub struct TestIntegration {
    #[structopt(long, help="The id of the organization",  parse(try_from_str = parse_org_id), default_value = "")]
    pub organization_id: OrgId,
    #[structopt(long, help="The id of the project",  parse(try_from_str = parse_project_id), default_value = "")]
    pub project_id: esc_api::resources::ProjectId,
    #[structopt(long, help = "The id of the integration")]
    pub integration_id: String,
}

#[derive(Debug, StructOpt)]
#[structopt(about = "updates a integration")]
pub struct UpdateIntegration {
    #[structopt(long, help="The id of the organization",  parse(try_from_str = parse_org_id), default_value = "")]
    pub organization_id: OrgId,
    #[structopt(long, help="The id of the project",  parse(try_from_str = parse_project_id), default_value = "")]
    pub project_id: esc_api::resources::ProjectId,
    #[structopt(long, help = "The id of the integration")]
    pub integration_id: String,
    #[structopt(subcommand)]
    pub data: Option<UpdateIntegrationData>,
    #[structopt(long)]
    pub description: Option<String>,
}

#[derive(Debug, StructOpt)]
#[structopt()]
pub struct UpdateIntegrationData {
    #[structopt(long, help = "API key used with the Ops Genie integration API")]
    pub api_key: Option<String>,
    #[structopt(long, help = "Slack Channel to send messages to")]
    pub channel_id: Option<String>,
    #[structopt(long, help = "API token for the Slack bot")]
    pub token: Option<String>,
}

lazy_static! {
    static ref PROVIDERS: HashMap<&'static str, esc_api::infra::Provider> = {
        let mut map = HashMap::new();
        map.insert("aws", esc_api::infra::Provider::Aws);
        map.insert("gcp", esc_api::infra::Provider::Gcp);
        map.insert("azure", esc_api::infra::Provider::Azure);
        map
    };
}

lazy_static! {
    static ref CONTEXT_PROP_NAMES: HashMap<&'static str, ProfilePropName> = {
        let mut map = HashMap::new();
        map.insert("project-id", ProfilePropName::ProjectId);
        map.insert("org-id", ProfilePropName::OrgId);
        map.insert("api-base-url", ProfilePropName::ApiBaseUrl);
        map.insert("fmt", ProfilePropName::Fmt);
        map
    };
}

lazy_static! {
    static ref CLUSTER_TOPOLOGIES: HashMap<&'static str, esc_api::mesdb::Topology> = {
        let mut map = HashMap::new();
        map.insert("single-node", esc_api::mesdb::Topology::SingleNode);
        map.insert(
            "three-node-multi-zone",
            esc_api::mesdb::Topology::ThreeNodeMultiZone,
        );
        map
    };
}

lazy_static! {
    static ref CLUSTER_PROJECTION_LEVELS: HashMap<&'static str, esc_api::mesdb::ProjectionLevel> = {
        let mut map = HashMap::new();
        map.insert("off", esc_api::mesdb::ProjectionLevel::Off);
        map.insert("system", esc_api::mesdb::ProjectionLevel::System);
        map.insert("user", esc_api::mesdb::ProjectionLevel::User);
        map
    };
}

fn parse_output_format(src: &str) -> Result<OutputFormat, String> {
    if src.trim().is_empty() {
        let profile_opt = crate::config::SETTINGS.get_current_profile();

        if let Some(value) = profile_opt.and_then(|p| p.output_format.as_ref()) {
            return Ok(value.clone());
        }
        return Ok(OutputFormat::Cli);
    }
    OutputFormat::from_str(src)
}

fn parse_org_id(src: &str) -> Result<esc_api::resources::OrganizationId, String> {
    if src.trim().is_empty() {
        let profile_opt = crate::config::SETTINGS.get_current_profile();

        if let Some(value) = profile_opt.and_then(|p| p.org_id.as_ref()) {
            return Ok(value.clone());
        }

        return Err("Not provided and you don't have an org-id property set in the [context] section of your settings.toml file".to_string());
    }

    Ok(esc_api::resources::OrganizationId(src.to_string()))
}

fn parse_project_id(src: &str) -> Result<esc_api::resources::ProjectId, String> {
    if src.trim().is_empty() {
        let profile_opt = crate::config::SETTINGS.get_current_profile();

        if let Some(value) = profile_opt.and_then(|p| p.project_id.as_ref()) {
            return Ok(value.clone());
        }

        return Err("Not provided and you don't have an project-id property set in the [context] section of your settings.toml file".to_string());
    }

    Ok(esc_api::resources::ProjectId(src.to_string()))
}

fn parse_acl_id(src: &str) -> Result<esc_api::infra::AclId, String> {
    Ok(esc_api::infra::AclId(src.to_string()))
}

fn parse_network_id(src: &str) -> Result<esc_api::infra::NetworkId, String> {
    Ok(esc_api::infra::NetworkId(src.to_string()))
}

fn parse_group_id(src: &str) -> Result<esc_api::GroupId, String> {
    Ok(esc_api::GroupId(src.to_string()))
}

fn parse_peering_id(src: &str) -> Result<esc_api::PeeringId, String> {
    Ok(esc_api::PeeringId(src.to_string()))
}

fn parse_cluster_id(src: &str) -> Result<esc_api::ClusterId, String> {
    Ok(esc_api::ClusterId(src.to_string()))
}

fn parse_backup_id(src: &str) -> Result<esc_api::BackupId, String> {
    Ok(esc_api::BackupId(src.to_string()))
}

fn parse_job_id(src: &str) -> Result<esc_api::JobId, String> {
    Ok(esc_api::JobId(src.to_string()))
}

fn parse_policy_id(src: &str) -> Result<esc_api::PolicyId, String> {
    Ok(esc_api::PolicyId(src.to_string()))
}

fn parse_member_id(src: &str) -> Result<esc_api::MemberId, String> {
    Ok(esc_api::MemberId(src.to_string()))
}

fn parse_provider(src: &str) -> Result<esc_api::infra::Provider, String> {
    parse_enum(&PROVIDERS, src)
}

fn parse_context_prop_name(src: &str) -> Result<ProfilePropName, String> {
    parse_enum(&CONTEXT_PROP_NAMES, src)
}

fn parse_topology(src: &str) -> Result<esc_api::mesdb::Topology, String> {
    parse_enum(&CLUSTER_TOPOLOGIES, src)
}

fn parse_projection_level(src: &str) -> Result<esc_api::mesdb::ProjectionLevel, String> {
    parse_enum(&CLUSTER_PROJECTION_LEVELS, src)
}

fn parse_cidr_input(s: &str) -> Result<esc_api::infra::AclCidrBlock, String> {
    if s.contains(',') {
        let (cidr, comment) = s
            .split_once(',')
            .ok_or(format!("Invalid CIDR input: {}", s))?;
        let cidr = cidr
            .parse::<cidr::Ipv4Cidr>()
            .map_err(|e| format!("Invalid CIDR input: {}", e))?;
        Ok(esc_api::infra::AclCidrBlock {
            address: cidr_to_string(cidr),
            comment: Some(comment.to_string()),
        })
    } else {
        let cidr = s
            .parse::<cidr::Ipv4Cidr>()
            .map_err(|e| format!("Invalid CIDR input: {}", e))?;
        Ok(esc_api::infra::AclCidrBlock {
            address: cidr_to_string(cidr),
            comment: None,
        })
    }
}

// When a CIDR is a host address, the output of Ipv4Cidr::to_string() is a
// single IP address. This is used to ensure that the output remains in CIDR
// notation.
fn cidr_to_string(cidr: cidr::Ipv4Cidr) -> String {
    if cidr.is_host_address() {
        format!("{}/32", cidr)
    } else {
        cidr.to_string()
    }
}

fn parse_enum<A: Clone>(env: &'static HashMap<&'static str, A>, src: &str) -> Result<A, String> {
    match env.get(src) {
        Some(p) => Ok(p.clone()),
        None => {
            let supported: Vec<&&str> = env.keys().collect();
            Err(format!(
                "Unsupported value: \"{}\". Supported values: {:?}",
                src, supported
            ))
        }
    }
}

fn parse_email(src: &str) -> Result<String, String> {
    if validator::validate_email(src) {
        return Ok(src.to_string());
    }

    Err("Invalid email".to_string())
}

fn parse_invite_id(src: &str) -> Result<esc_api::access::InviteId, String> {
    Ok(esc_api::access::InviteId(src.to_string()))
}

fn parse_cidr(src: &str) -> Result<cidr::Ipv4Cidr, cidr::NetworkParseError> {
    src.parse::<cidr::Ipv4Cidr>()
}

struct Printer {
    pub render_in_json: bool,
    pub render_as_v1: bool,
}

impl Printer {
    pub fn print<A: std::fmt::Debug + Serialize + v1::ToV1>(
        &self,
        value: A,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.render_as_v1 {
            let value = value.to_v1();
            if self.render_in_json {
                serde_json::to_writer_pretty(std::io::stdout(), &value)?;
            } else {
                println!("{:?}", value);
            }
        }
        Ok(())
    }

    pub fn print_json_only<A: std::fmt::Debug + Serialize>(
        &self,
        value: A,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.render_as_v1 {
            serde_json::to_writer_pretty(std::io::stdout(), &value)?;
        }
        Ok(())
    }
}

pub struct StaticAuthorization {
    pub authorization_header: String,
}

impl esc_api::Authorization for StaticAuthorization {
    fn authorization_header(&self) -> String {
        self.authorization_header.clone()
    }

    fn refresh(&mut self) -> bool {
        false
    }
}

async fn get_token(
    token_config: esc_api::TokenConfig,
    refresh_token: Option<String>,
    noninteractive: bool,
) -> Result<esc_api::Token, Box<dyn std::error::Error>> {
    let client = build_http_client();
    match refresh_token {
        Some(refresh_token) => {
            let otp_prompt: Option<esc_client_base::identity::operations::OtpPrompt> =
                match noninteractive {
                    true => None,
                    false => Some(esc_client_store::prompt_for_otp),
                };
            let refreshed_token = esc_client_base::identity::operations::refresh(
                &client,
                &token_config,
                &refresh_token,
                otp_prompt,
            )
            .await?;
            Ok(refreshed_token)
        }
        None => {
            let mut store = esc_client_store::token_store(token_config).await?;
            let token = store.access(&client, noninteractive).await?;
            Ok(token)
        }
    }
}

struct TrafficSpy {
    verbose: bool,
}

impl esc_api::RequestObserver for TrafficSpy {
    fn on_request(&self, method: &str, url: &str, body: &str) {
        if self.verbose {
            println!("{} {}", method, url);
        }
        if !body.is_empty() {
            println!("{}", body);
        }
    }

    fn on_response(&self, status: &str, body: &str) {
        if self.verbose || !(status.len() == 3 && status.starts_with('2')) {
            println!("status: {}", status);
        }
        if !body.is_empty() {
            println!("{}", body);
        };
    }
}

struct ClientBuilder {
    base_url: String,
    observer: Option<Arc<dyn esc_api::RequestObserver + Send + Sync>>,
    refresh_token: Option<String>,
    token_config: esc_api::TokenConfig,
    noninteractive: bool,
}

impl ClientBuilder {
    pub async fn create(self) -> Result<esc_api::Client, Box<dyn std::error::Error>> {
        let token = get_token(self.token_config, self.refresh_token, self.noninteractive).await?;
        let authorization = StaticAuthorization {
            authorization_header: token.authorization_header(),
        };
        let sender = esc_api::RequestSender {
            client: build_http_client(),
            observer: self.observer,
        };
        let client = esc_api::Client {
            authorization: std::sync::Arc::new(authorization),
            base_url: self.base_url,
            sender,
        };
        Ok(client)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let clap_app = Opt::clap();
    let opt = Opt::from_clap(&clap_app.clone().get_matches());

    let base_url = config::SETTINGS
        .get_current_profile()
        .and_then(|profile| {
            profile.api_base_url.as_ref().map(|url| {
                format!(
                    "{}://{}",
                    url.scheme(),
                    url.host_str().expect("Pre-validated it has a host")
                )
            })
        })
        .unwrap_or_else(|| constants::ES_CLOUD_API_URL.to_string());

    let observer: Option<Arc<dyn esc_api::RequestObserver + Send + Sync>> =
        if !opt.output_format.is_v1() {
            Some(Arc::new(TrafficSpy {
                verbose: matches!(opt.output_format, OutputFormat::ApiVerbose),
            }))
        } else {
            None
        };

    let printer = Printer {
        render_in_json: match opt.output_format {
            OutputFormat::CliJson => true,
            _ => opt.render_in_json,
        },
        render_as_v1: opt.output_format.is_v1(),
    };

    config::Settings::configure().await?;

    if opt.debug {
        std::env::set_var("RUST_LOG", "esc_api=debug,esc=debug");
        env_logger::init();
    }

    // Create the token config
    let mut token_config = esc_api::TokenConfig::default();
    // If the user has specified additional token config settings, change them here.
    // No typical users will ever need to do this, so it's only accessible if the
    // config file is changed directly.
    let profile_opt = crate::config::SETTINGS.get_current_profile();
    if let Some(token_opts) = profile_opt.and_then(|p| p.token_config.as_ref()) {
        if let Some(value) = &token_opts.audience {
            token_config.audience = value.clone();
        }
        if let Some(value) = &token_opts.client_id {
            token_config.client_id = value.clone();
        }
        if let Some(value) = &token_opts.identity_url {
            token_config.identity_url = value.clone();
        }
        if let Some(value) = &token_opts.public_key {
            token_config.public_key = value.clone();
        }
    }

    let client_builder = ClientBuilder {
        base_url,
        observer,
        refresh_token: opt.refresh_token.clone(),
        token_config: token_config.clone(),
        noninteractive: opt.noninteractive,
    };

    let silence_errors = !opt.output_format.is_v1();
    let result = call_api(clap_app, opt, client_builder, printer, token_config).await;
    if !silence_errors {
        result
    } else if let Err(err) = result {
        if let Some(esc_api::Error::ApiResponse(resp)) = err.downcast_ref::<esc_api::Error>() {
            if !resp.status_code.is_success() {
                // The traffic observer has already shown the error to the user, so don't show
                // additional error information.
                std::process::exit(1);
            }
        }
        Err(err)
    } else {
        result
    }
}

async fn call_api<'a, 'b>(
    mut clap_app: clap::App<'a, 'b>,
    opt: Opt,
    client_builder: ClientBuilder,
    printer: Printer,
    token_config: esc_api::TokenConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    match opt.cmd {
        Command::Access(access) => match access.access_command {
            AccessCommand::Groups(groups) => match groups.groups_command {
                GroupsCommand::Create(params) => {
                    let client = client_builder.create().await?;
                    let create_params = esc_api::access::CreateGroupRequest {
                        name: params.name,
                        members: Some(
                            params
                                .members
                                .iter()
                                .map(|m| esc_api::access::MemberId(m.clone()))
                                .collect(),
                        ),
                    };
                    let resp = esc_api::access::create_group(&client, params.org_id, create_params)
                        .await?;

                    printer.print(resp)?;
                }

                GroupsCommand::Update(params) => {
                    let client = client_builder.create().await?;
                    let body = esc_api::access::UpdateGroupRequest {
                        members: params.members,
                        name: params.name,
                    };
                    esc_api::access::update_group(&client, params.org_id, params.id, body).await?;
                }

                GroupsCommand::Get(params) => {
                    let client = client_builder.create().await?;
                    let resp =
                        esc_api::access::get_group(&client, params.org_id, params.id).await?;
                    printer.print(resp)?;
                }

                GroupsCommand::Delete(params) => {
                    let client = client_builder.create().await?;
                    esc_api::access::delete_group(&client, params.org_id, params.id).await?;
                }

                GroupsCommand::List(params) => {
                    let client = client_builder.create().await?;
                    let linked_resource = None; // TODO: add this as a parameter
                    let resp =
                        esc_api::access::list_groups(&client, params.org_id, linked_resource)
                            .await?;
                    printer.print(resp)?;
                }
            },

            AccessCommand::Invites(invites) => match invites.invites_command {
                InvitesCommand::Create(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::access::create_invite(
                        &client,
                        params.org_id,
                        esc_api::access::CreateInviteRequest {
                            groups: params.group,
                            user_email: params.email,
                        },
                    )
                    .await?;

                    printer.print(resp)?;
                }

                InvitesCommand::Resend(params) => {
                    let client = client_builder.create().await?;
                    esc_api::access::resend_invite(
                        &client,
                        params.org_id,
                        esc_api::access::ResendInviteRequest { id: params.id },
                    )
                    .await?;
                }

                InvitesCommand::Delete(params) => {
                    let client = client_builder.create().await?;
                    esc_api::access::delete_invite(&client, params.org_id, params.id).await?;
                }

                InvitesCommand::List(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::access::list_invites(&client, params.org_id).await?;
                    printer.print(resp)?;
                }
            },

            AccessCommand::Tokens(tokens) => match tokens.tokens_command {
                TokensCommand::Create(params) => {
                    let client = build_http_client();
                    let mut store = esc_client_store::token_store(token_config).await?;

                    match client_builder.noninteractive {
                        true => {
                            println!("--noninteractive mode set, cannot prompt for password");
                            std::process::exit(-1)
                        }
                        false => {
                            let token = match params.email {
                                Some(email) => match params.unsafe_password {
                                    Some(password) => {
                                        store.create_token(&client, email, password).await
                                    }
                                    None => {
                                        store
                                            .create_token_from_prompt_password_only(&client, email)
                                            .await
                                    }
                                },
                                None => store.create_token_from_prompt(&client).await,
                            }?;
                            println!("{}", token.refresh_token().unwrap().as_str());
                        }
                    }
                }
                TokensCommand::Display(_params) => {
                    let store = esc_client_store::token_store(token_config).await?;

                    let token = store.show().await?;
                    if let Some(token) = token {
                        println!("{}", token.refresh_token().unwrap());
                    } else {
                        println!("No active refresh token");
                        std::process::exit(-1)
                    }
                }
            },

            AccessCommand::Policies(policies) => match policies.policies_command {
                PoliciesCommand::Create(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::access::create_policy(
                        &client,
                        params.org_id,
                        esc_api::access::CreatePolicyRequest {
                            policy: esc_api::access::CreatePolicy {
                                actions: utils::actions_from_str_vec(params.actions),
                                effect: utils::effect_from_str(&params.effect),
                                name: params.name,
                                resources: params.resources,
                                subjects: params.subjects,
                            },
                        },
                    )
                    .await?;
                    printer.print(resp)?;
                }

                PoliciesCommand::Update(params) => {
                    let client = client_builder.create().await?;
                    esc_api::access::update_policy(
                        &client,
                        params.org_id,
                        params.policy,
                        esc_api::access::UpdatePolicyRequest {
                            policy: esc_api::access::UpdatePolicy {
                                actions: utils::actions_from_str_vec(params.actions),
                                effect: utils::effect_from_str(&params.effect),
                                name: params.name,
                                resources: params.resources,
                                subjects: params.subjects,
                            },
                        },
                    )
                    .await?;
                }

                PoliciesCommand::Delete(params) => {
                    let client = client_builder.create().await?;
                    esc_api::access::delete_policy(&client, params.org_id, params.policy).await?;
                }

                PoliciesCommand::Get(params) => {
                    let client = client_builder.create().await?;
                    let resp =
                        esc_api::access::get_policy(&client, params.org_id, params.policy).await?;
                    printer.print(resp)?;
                }

                PoliciesCommand::List(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::access::list_policies(&client, params.org_id).await?;
                    printer.print(resp)?;
                }
            },

            AccessCommand::Members(members) => match members.members_command {
                MembersCommand::List(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::access::list_members(&client, params.org_id).await?;
                    printer.print(resp)?;
                }

                MembersCommand::Get(params) => {
                    let client = client_builder.create().await?;
                    let resp =
                        esc_api::access::get_member(&client, params.org_id, params.id).await?;
                    printer.print(resp)?;
                }

                MembersCommand::Update(params) => {
                    let client = client_builder.create().await?;
                    esc_api::access::update_member(
                        &client,
                        params.org_id,
                        params.id,
                        esc_api::access::UpdateMemberRequest {
                            active: params.active,
                        },
                    )
                    .await?;
                }

                MembersCommand::Delete(params) => {
                    let client = client_builder.create().await?;
                    esc_api::access::delete_member(&client, params.org_id, params.id).await?;
                }
            },
        },

        Command::Audit(aud) => match aud.audit_command {
            AuditCommand::Organization(orgs) => match orgs.organization_command {
                AuditOrganizationCommand::Get(params) => {
                    let mut before = "".to_string();
                    let mut after = "".to_string();
                    let mut limit = "".to_string();

                    match params.before {
                        Some(x) => before = x,
                        None => {}
                    }
                    match params.after {
                        Some(x) => after = x,
                        None => {}
                    }
                    match params.limit {
                        Some(x) => limit = x,
                        None => {}
                    }
                    let client = client_builder.create().await?;
                    let resp = esc_api::audit::get_audit_by_org(
                        &client,
                        params.org_id,
                        before,
                        after,
                        limit,
                    )
                    .await?;
                    printer.print(resp)?;
                }
            },
            AuditCommand::User(user) => match user.user_command {
                AuditUserCommand::Get(params) => {
                    let mut org_id = "".to_string();
                    let mut before = "".to_string();
                    let mut after = "".to_string();
                    let mut limit = "".to_string();

                    match params.before {
                        Some(x) => before = x,
                        None => {}
                    }
                    match params.after {
                        Some(x) => after = x,
                        None => {}
                    }
                    match params.limit {
                        Some(x) => limit = x,
                        None => {}
                    }
                    match params.org_id {
                        Some(x) => org_id = x,
                        None => {}
                    }
                    let client = client_builder.create().await?;
                    let resp =
                        esc_api::audit::get_audit_by_user(&client, org_id, before, after, limit)
                            .await?;
                    printer.print(resp)?;
                }
            },
        },

        Command::Infra(infra) => match infra.infra_command {
            InfraCommand::Acls(acls) => match acls.acls_command {
                AclsCommand::Create(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::infra::create_acl(
                        &client,
                        params.org_id,
                        params.project_id,
                        esc_api::infra::CreateAclRequest {
                            cidr_blocks: params.cidr_blocks,
                            description: params.description,
                        },
                    )
                    .await?;
                    printer.print_json_only(resp)?;
                }
                AclsCommand::Delete(params) => {
                    let client = client_builder.create().await?;
                    esc_api::infra::delete_acl(
                        &client,
                        params.org_id,
                        params.project_id,
                        params.id,
                    )
                    .await?;
                }
                AclsCommand::Get(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::infra::get_acl(
                        &client,
                        params.org_id,
                        params.project_id,
                        params.id,
                    )
                    .await?;
                    printer.print_json_only(resp)?;
                }
                AclsCommand::List(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::infra::list_acls(&client, params.org_id, params.project_id)
                        .await?;
                    printer.print_json_only(resp)?;
                }
                AclsCommand::Update(params) => {
                    let client = client_builder.create().await?;
                    esc_api::infra::update_acl(
                        &client,
                        params.org_id,
                        params.project_id,
                        params.id,
                        esc_api::infra::UpdateAclRequest {
                            cidr_blocks: Some(params.cidr_blocks),
                            description: params.description,
                        },
                    )
                    .await?;
                }
            },
            InfraCommand::Networks(networks) => match networks.networks_command {
                NetworksCommand::Create(params) => {
                    let cidr_block = params.cidr_block.map(|cidr| cidr.to_string());
                    let client = client_builder.create().await?;
                    let resp = esc_api::infra::create_network(
                        &client,
                        params.org_id,
                        params.project_id,
                        esc_api::infra::CreateNetworkRequest {
                            cidr_block,
                            description: params.description,
                            provider: params.provider.to_string(),
                            region: params.region,
                            public_access: params.public_access,
                        },
                    )
                    .await?;
                    printer.print(resp)?;
                }

                NetworksCommand::Update(params) => {
                    let client = client_builder.create().await?;
                    esc_api::infra::update_network(
                        &client,
                        params.org_id,
                        params.project_id,
                        params.id,
                        esc_api::infra::UpdateNetworkRequest {
                            description: params.description,
                        },
                    )
                    .await?;
                }

                NetworksCommand::Delete(params) => {
                    let client = client_builder.create().await?;
                    esc_api::infra::delete_network(
                        &client,
                        params.org_id,
                        params.project_id,
                        params.id,
                    )
                    .await?;
                }

                NetworksCommand::Get(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::infra::get_network(
                        &client,
                        params.org_id,
                        params.project_id,
                        params.id,
                    )
                    .await?;
                    printer.print(resp)?;
                }

                NetworksCommand::List(params) => {
                    let client = client_builder.create().await?;
                    let resp =
                        esc_api::infra::list_networks(&client, params.org_id, params.project_id)
                            .await?;
                    printer.print(resp)?;
                }
            },

            InfraCommand::Peerings(peerings) => match peerings.peerings_command {
                PeeringsCommand::Create(params) => {
                    let client = client_builder.create().await?;
                    let result = esc_api::infra::create_peering(
                        &client,
                        params.org_id.clone(),
                        params.project_id.clone(),
                        esc_api::infra::CreatePeeringRequest {
                            description: params.description,
                            network_id: params.network_id.clone(),
                            peer_account_id: params.peer_account_id.clone(),
                            peer_network_id: params.peer_network_id.clone(),
                            peer_network_region: params.peer_network_region,
                            routes: params.routes,
                        },
                    )
                    .await;

                    if let Err(_err) = result {
                        let network = esc_api::infra::get_network(
                            &client,
                            params.org_id.clone(),
                            params.project_id.clone(),
                            params.network_id.clone(),
                        )
                        .await?;

                        let resp = esc_api::infra::create_peering_commands(
                            &client,
                            params.org_id,
                            params.project_id,
                            esc_api::infra::CreatePeeringCommandsRequest {
                                provider: network.network.provider,
                                peer_account_id: params.peer_account_id,
                                peer_network_id: params.peer_network_id,
                            },
                        )
                        .await?;

                        if opt.render_in_json {
                            printer.print(resp)?;
                        } else {
                            println!("Upstream provider requires configuration.");
                            for command in resp.commands {
                                println!();
                                println!("{}:", command.title);
                                println!("{}", command.value);
                            }
                        }
                    }
                }

                PeeringsCommand::Update(params) => {
                    let client = client_builder.create().await?;
                    esc_api::infra::update_peering(
                        &client,
                        params.org_id,
                        params.project_id,
                        params.id,
                        esc_api::infra::UpdatePeeringRequest {
                            description: params.description,
                        },
                    )
                    .await?;
                }

                PeeringsCommand::Delete(params) => {
                    let client = client_builder.create().await?;
                    esc_api::infra::delete_peering(
                        &client,
                        params.org_id,
                        params.project_id,
                        params.id,
                    )
                    .await?;
                }

                PeeringsCommand::Get(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::infra::get_peering(
                        &client,
                        params.org_id,
                        params.project_id,
                        params.id,
                    )
                    .await?;
                    printer.print(resp)?;
                }

                PeeringsCommand::List(params) => {
                    let client = client_builder.create().await?;
                    let resp =
                        esc_api::infra::list_peerings(&client, params.org_id, params.project_id)
                            .await?;
                    printer.print(resp)?;
                }
            },
        },

        Command::Profiles(context) => match context.profiles_command {
            ProfilesCommand::Set(params) => {
                let mut settings = crate::config::SETTINGS.clone();
                let profile = settings.get_profile_mut(&params.profile);

                match params.name {
                    ProfilePropName::ProjectId => {
                        profile.project_id = Some(esc_api::resources::ProjectId(params.value));
                    }

                    ProfilePropName::OrgId => {
                        profile.org_id = Some(esc_api::resources::OrganizationId(params.value));
                    }

                    ProfilePropName::ApiBaseUrl => {
                        let url = config::parse_url(params.value.as_str())?;
                        profile.api_base_url = Some(url);
                    }

                    ProfilePropName::Fmt => {
                        let fmt = OutputFormat::from_str(params.value.as_str())?;
                        profile.output_format = Some(fmt);
                    }
                }

                settings.persist().await?;
            }

            ProfilesCommand::Get(params) => {
                if let Some(profile) = crate::config::SETTINGS.get_profile(&params.profile) {
                    if let Some(name) = params.name {
                        match name {
                            ProfilePropName::ProjectId => {
                                // TODO: not sure why ProjectID ever had a default that gave it a blank string.
                                // But that's what "default" used to be here. It would be ideal to make this
                                // Option<ProjectId>.
                                let default = esc_api::resources::ProjectId("".to_string());
                                let value = profile.project_id.as_ref().unwrap_or(&default);
                                serde_json::to_writer_pretty(std::io::stdout(), value)?;
                            }

                            ProfilePropName::OrgId => {
                                // TODO: same issue, not sure why it works this way
                                let default = esc_api::resources::OrganizationId("".to_string());
                                let value = profile.org_id.as_ref().unwrap_or(&default);
                                serde_json::to_writer_pretty(std::io::stdout(), value)?;
                            }

                            ProfilePropName::ApiBaseUrl => {
                                if let Some(url) = profile.api_base_url.as_ref() {
                                    serde_json::to_writer_pretty(std::io::stdout(), url.as_str())?;
                                }
                            }

                            ProfilePropName::Fmt => {
                                if let Some(fmt) = profile.output_format.as_ref() {
                                    serde_json::to_writer_pretty(std::io::stdout(), fmt.as_str())?;
                                }
                            }
                        }
                    } else {
                        serde_json::to_writer_pretty(std::io::stdout(), profile)?;
                    }
                }
            }

            ProfilesCommand::List => {
                serde_json::to_writer_pretty(std::io::stdout(), &crate::config::SETTINGS.profiles)?;
            }

            ProfilesCommand::Delete(params) => {
                let mut settings = crate::config::SETTINGS.clone();
                let profile = settings.get_profile_mut(&params.profile);

                match params.name {
                    ProfilePropName::ProjectId => {
                        profile.project_id = None;
                    }

                    ProfilePropName::OrgId => {
                        profile.org_id = None;
                    }

                    ProfilePropName::ApiBaseUrl => {
                        profile.api_base_url = None;
                    }

                    ProfilePropName::Fmt => {
                        profile.output_format = None;
                    }
                }

                settings.persist().await?;
            }

            ProfilesCommand::Default(default) => match default.default_command {
                ProfileDefaultCommand::Get(_) => {
                    match crate::config::SETTINGS.default_profile.as_ref() {
                        Some(value) => serde_json::to_writer_pretty(std::io::stdout(), value)?,
                        _ => {
                            println!(
                                "No default profile set\n\n\
                            To set a default profile use:\n\n\
                            esc profiles default set <profile_name>"
                            );
                            std::process::exit(-1)
                        }
                    }
                }

                ProfileDefaultCommand::Set(params) => {
                    let mut settings = crate::config::SETTINGS.clone();
                    settings.default_profile = Some(params.value);
                    settings.persist().await?;
                }
            },
        },

        Command::Resources(res) => match res.resources_command {
            ResourcesCommand::Organizations(orgs) => match orgs.organizations_command {
                OrganizationsCommand::Create(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::resources::create_organization(
                        &client,
                        esc_api::resources::CreateOrganizationRequest { name: params.name },
                    )
                    .await?;
                    printer.print(resp)?;
                }

                OrganizationsCommand::Update(params) => {
                    let client = client_builder.create().await?;
                    esc_api::resources::update_organization(
                        &client,
                        params.id,
                        esc_api::resources::UpdateOrganizationRequest { name: params.name },
                    )
                    .await?;
                }

                OrganizationsCommand::Delete(params) => {
                    let client = client_builder.create().await?;
                    esc_api::resources::delete_organization(&client, params.id).await?;
                }

                OrganizationsCommand::Get(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::resources::get_organization(&client, params.id).await?;
                    printer.print(resp)?;
                }

                OrganizationsCommand::List(_) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::resources::list_organizations(&client).await?;
                    printer.print(resp)?;
                }

                OrganizationsCommand::GetMfaStatus(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::resources::get_mfa_status(&client, params.id).await?;
                    printer.print(resp)?;
                }

                OrganizationsCommand::UpdateMfaStatus(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::resources::update_mfa(
                        &client,
                        params.id,
                        MfaStatus {
                            mfa_enabled: params.enabled,
                        },
                    )
                    .await?;
                    printer.print(resp)?;
                }
            },

            ResourcesCommand::Projects(projs) => match projs.projects_command {
                ProjectsCommand::Create(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::resources::create_project(
                        &client,
                        params.org_id,
                        esc_api::resources::CreateProjectRequest { name: params.name },
                    )
                    .await?;
                    printer.print(resp)?;
                }

                ProjectsCommand::Update(params) => {
                    let client = client_builder.create().await?;
                    esc_api::resources::update_project(
                        &client,
                        params.org_id,
                        params.id,
                        esc_api::resources::UpdateProjectRequest { name: params.name },
                    )
                    .await?;
                }

                ProjectsCommand::Get(params) => {
                    let client = client_builder.create().await?;
                    let resp =
                        esc_api::resources::get_project(&client, params.org_id, params.id).await?;
                    printer.print(resp)?;
                }

                ProjectsCommand::Delete(params) => {
                    let client = client_builder.create().await?;
                    esc_api::resources::delete_project(&client, params.org_id, params.id).await?;
                }

                ProjectsCommand::List(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::resources::list_projects(&client, params.org_id).await?;
                    printer.print(resp)?;
                }
            },
        },

        Command::Mesdb(mesdb) => {
            match mesdb.mesdb_command {
                MesdbCommand::SharedClusters(shared_clusters) => {
                    match shared_clusters.shared_clusters_command {
                        SharedClustersCommand::Create(params) => {
                            let client = client_builder.create().await?;
                            let resp = esc_api::mesdb::create_shared_cluster(
                                &client,
                                params.org_id,
                                params.project_id,
                                esc_api::mesdb::CreateSharedClusterDeploymentRequest {
                                    cluster: esc_api::mesdb::CreateSharedClusterRequest {
                                        name: params.name,
                                        provider: params.provider,
                                        region: params.region,
                                        deployment_tier: params.deployment_tier,
                                        mutual_tls_enabled: params.mutual_tls_enabled,
                                        projection_level: params.projection_level,
                                        server_version: params.server_version,
                                        topology: params.topology,
                                    },
                                    acl: esc_api::mesdb::Acl::ResourceIdentifier(
                                        esc_api::mesdb::ResourceIdentifier {
                                            id: params.acl_id.0,
                                        },
                                    ),
                                },
                            )
                            .await?;
                            printer.print(resp)?;
                        }

                        SharedClustersCommand::List(params) => {
                            let client = client_builder.create().await?;
                            let resp = esc_api::mesdb::list_shared_clusters(
                                &client,
                                params.org_id,
                                params.project_id,
                            )
                            .await?;
                            printer.print(resp)?;
                        }

                        SharedClustersCommand::Delete(params) => {
                            let client = client_builder.create().await?;
                            esc_api::mesdb::delete_shared_cluster(
                                &client,
                                params.org_id,
                                params.project_id,
                                params.id,
                            )
                            .await?;
                        }

                        SharedClustersCommand::Get(params) => {
                            let client = client_builder.create().await?;
                            let resp = esc_api::mesdb::get_shared_cluster(
                                &client,
                                params.org_id,
                                params.project_id,
                                params.id,
                            )
                            .await?;
                            printer.print(resp)?;
                        }
                    }
                }

                MesdbCommand::Clusters(clusters) => match clusters.clusters_command {
                    ClustersCommand::Create(params) => {
                        let client = client_builder.create().await?;
                        let resp = esc_api::mesdb::create_cluster(
                            &client,
                            params.org_id,
                            params.project_id,
                            esc_api::mesdb::CreateClusterRequest {
                                acl_id: params.acl_id,
                                description: params.description,
                                disk_iops: params.disk_iops,
                                disk_size_gb: params.disk_size_in_gb,
                                disk_throughput: params.disk_throughput,
                                disk_type: params.disk_type,
                                instance_type: params.instance_type,
                                network_id: params.network_id,
                                projection_level: params.projection_level,
                                server_version: params.server_version,
                                source_backup_id: params.source_backup_id,
                                source_backup_project_id: params.source_project_id,
                                source_node_index: None, // TODO: add source_node_index
                                topology: params.topology,
                                protected: params.protected,
                                public_access: params.public_access,
                                provider: params.provider,
                                region: params.region,
                            },
                        )
                        .await?;
                        printer.print(resp)?;
                    }

                    ClustersCommand::Get(params) => {
                        let client = client_builder.create().await?;
                        let resp = esc_api::mesdb::get_cluster(
                            &client,
                            params.org_id,
                            params.project_id,
                            params.id,
                        )
                        .await?;
                        printer.print(resp)?;
                    }

                    ClustersCommand::Delete(params) => {
                        let client = client_builder.create().await?;
                        esc_api::mesdb::delete_cluster(
                            &client,
                            params.org_id,
                            params.project_id,
                            params.id,
                        )
                        .await?;
                    }

                    ClustersCommand::Update(params) => {
                        let client = client_builder.create().await?;
                        esc_api::mesdb::update_cluster(
                            &client,
                            params.org_id,
                            params.project_id,
                            params.id,
                            esc_api::mesdb::UpdateClusterRequest {
                                acl_id: params.acl_id,
                                description: params.description,
                                protected: params.protected,
                            },
                        )
                        .await?;
                    }

                    ClustersCommand::List(params) => {
                        let client = client_builder.create().await?;
                        let resp = esc_api::mesdb::list_clusters(
                            &client,
                            params.org_id,
                            params.project_id,
                        )
                        .await?;
                        printer.print(resp)?;
                    }

                    ClustersCommand::Expand(params) => {
                        let client = client_builder.create().await?;
                        esc_api::mesdb::expand_cluster_disk(
                            &client,
                            params.org_id,
                            params.project_id,
                            params.id,
                            esc_api::mesdb::ExpandClusterDiskRequest {
                                disk_iops: params.disk_iops,
                                disk_size_gb: params.disk_size_in_gb,
                                disk_throughput: params.disk_throughput,
                                disk_type: params.disk_type,
                            },
                        )
                        .await?;
                    }

                    ClustersCommand::Resize(params) => {
                        let client = client_builder.create().await?;
                        esc_api::mesdb::resize_cluster(
                            &client,
                            params.org_id,
                            params.project_id,
                            params.id,
                            esc_api::mesdb::ResizeClusterRequest {
                                target_size: params.target_size,
                            },
                        )
                        .await?;
                    }

                    ClustersCommand::Stop(params) => {
                        let client = client_builder.create().await?;
                        esc_api::mesdb::stop_cluster(
                            &client,
                            params.org_id,
                            params.project_id,
                            params.id,
                        )
                        .await?;
                    }

                    ClustersCommand::Start(params) => {
                        let client = client_builder.create().await?;
                        esc_api::mesdb::start_cluster(
                            &client,
                            params.org_id,
                            params.project_id,
                            params.id,
                        )
                        .await?;
                    }

                    ClustersCommand::Upgrade(params) => {
                        let client = client_builder.create().await?;
                        esc_api::mesdb::upgrade_cluster(
                            &client,
                            params.org_id,
                            params.project_id,
                            params.id,
                            esc_api::mesdb::UpgradeClusterRequest {
                                target_tag: params.target_tag,
                            },
                        )
                        .await?;
                    }
                },
                MesdbCommand::Backups(clusters) => match clusters.backups_command {
                    BackupsCommand::Create(params) => {
                        let client = client_builder.create().await?;
                        let resp = esc_api::mesdb::create_backup(
                            &client,
                            params.org_id,
                            params.project_id,
                            esc_api::mesdb::CreateBackupRequest {
                                description: params.description,
                                source_cluster_id: params.source_cluster_id,
                            },
                        )
                        .await?;
                        printer.print(resp)?;
                    }

                    BackupsCommand::Get(params) => {
                        let client = client_builder.create().await?;
                        let resp = esc_api::mesdb::get_backup(
                            &client,
                            params.org_id,
                            params.project_id,
                            params.id,
                        )
                        .await?;
                        printer.print(resp)?;
                    }

                    BackupsCommand::Delete(params) => {
                        let client = client_builder.create().await?;
                        esc_api::mesdb::delete_backup(
                            &client,
                            params.org_id,
                            params.project_id,
                            params.id,
                        )
                        .await?;
                    }

                    BackupsCommand::List(params) => {
                        let client = client_builder.create().await?;
                        let resp =
                            esc_api::mesdb::list_backups(&client, params.org_id, params.project_id)
                                .await?;
                        printer.print(resp)?;
                    }
                },
            }
        }

        Command::Orchestrate(orchestrate) => match orchestrate.orchestrate_command {
            OrchestrateCommand::Jobs(jobs) => match jobs.jobs_command {
                JobsCommand::Create(params) => {
                    let client = client_builder.create().await?;
                    let data = match params.job_type {
                        CreateJobType::ScheduledBackup(args) => {
                            esc_api::orchestrate::JobData::ScheduledBackup(
                                esc_api::orchestrate::ScheduledBackupData {
                                    cluster_id: args.cluster_id,
                                    description: args.description,
                                    max_backup_count: args.max_backup_count,
                                },
                            )
                        }
                    };
                    let resp = esc_api::orchestrate::create_job(
                        &client,
                        params.org_id,
                        params.project_id,
                        esc_api::orchestrate::CreateJobRequest {
                            data,
                            description: params.description,
                            schedule: params.schedule,
                        },
                    )
                    .await?;
                    printer.print(resp)?;
                }

                JobsCommand::Get(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::orchestrate::get_job(
                        &client,
                        params.org_id,
                        params.project_id,
                        params.id,
                    )
                    .await?;
                    printer.print(resp)?;
                }

                JobsCommand::Delete(params) => {
                    let client = client_builder.create().await?;
                    esc_api::orchestrate::delete_job(
                        &client,
                        params.org_id,
                        params.project_id,
                        params.id,
                    )
                    .await?;
                }

                JobsCommand::List(params) => {
                    let client = client_builder.create().await?;
                    let resp =
                        esc_api::orchestrate::list_jobs(&client, params.org_id, params.project_id)
                            .await?;
                    printer.print(resp)?;
                }
            },
            OrchestrateCommand::History(history) => match history.history_command {
                HistoryCommand::List(params) => {
                    let client = client_builder.create().await?;
                    let resp = esc_api::orchestrate::list_history(
                        &client,
                        params.org_id,
                        params.project_id,
                        params.job_id,
                    )
                    .await?;
                    printer.print(resp)?;
                }
            },
        },

        Command::Integrations(cmd) => match cmd.integration_command {
            IntegrationsCommand::List(params) => {
                let client = client_builder.create().await?;
                let resp = esc_api::integrate::list_integrations(
                    &client,
                    params.organization_id,
                    params.project_id,
                )
                .await?;
                printer.print(resp)?;
            }
            IntegrationsCommand::Create(params) => {
                let client = client_builder.create().await?;
                let data: esc_api::integrate::CreateIntegrationData = match params.data {
                    CreateIntegrationData::OpsGenie(args) => {
                        esc_api::integrate::CreateIntegrationData::OpsGenie(
                            esc_api::integrate::CreateOpsGenieIntegrationData {
                                api_key: args.api_key,
                                source: None,
                            },
                        )
                    }
                    CreateIntegrationData::Slack(args) => {
                        esc_api::integrate::CreateIntegrationData::Slack(
                            esc_api::integrate::CreateSlackIntegrationData {
                                channel_id: args.channel_id,
                                source: args.source,
                                token: args.token,
                            },
                        )
                    }
                };
                let resp = esc_api::integrate::create_integration(
                    &client,
                    params.organization_id,
                    params.project_id,
                    esc_api::integrate::CreateIntegrationRequest {
                        data,
                        description: params.description,
                    },
                )
                .await?;
                printer.print(resp)?;
            }
            IntegrationsCommand::Delete(params) => {
                let client = client_builder.create().await?;
                esc_api::integrate::delete_integration(
                    &client,
                    params.organization_id,
                    params.project_id,
                    esc_api::integrate::IntegrationId(params.integration_id),
                )
                .await?;
            }
            IntegrationsCommand::Get(params) => {
                let client = client_builder.create().await?;
                let resp = esc_api::integrate::get_integration(
                    &client,
                    params.organization_id,
                    params.project_id,
                    esc_api::integrate::IntegrationId(params.integration_id),
                )
                .await?;
                printer.print(resp)?;
            }
            IntegrationsCommand::Update(params) => {
                use esc_api::integrate::*;
                // TODO: rework this. It's probably saner to force the user to say the type of sink they're updating
                let data: Option<UpdateIntegrationData> = match params.data {
                    Some(data) => match data.api_key {
                        Some(api_key) => {
                            Some(UpdateIntegrationData::UpdateOpsGenieIntegrationData(
                                UpdateOpsGenieIntegrationData {
                                    api_key: Some(api_key),
                                },
                            ))
                        }
                        None => {
                            if data.channel_id.is_some() || data.token.is_some() {
                                Some(UpdateIntegrationData::UpdateSlackIntegrationData(
                                    UpdateSlackIntegrationData {
                                        channel_id: data.channel_id,
                                        token: data.token,
                                    },
                                ))
                            } else {
                                None
                            }
                        }
                    },
                    None => None,
                };

                let client = client_builder.create().await?;
                esc_api::integrate::update_integration(
                    &client,
                    params.organization_id,
                    params.project_id,
                    esc_api::integrate::IntegrationId(params.integration_id),
                    UpdateIntegrationRequest {
                        description: params.description,
                        data,
                    },
                )
                .await?;
            }
            IntegrationsCommand::TestIntegration(params) => {
                let client = client_builder.create().await?;
                esc_api::integrate::test_integration(
                    &client,
                    params.organization_id,
                    params.project_id,
                    esc_api::integrate::IntegrationId(params.integration_id),
                )
                .await?;
            }
        },

        Command::GenerateBashCompletion => {
            // clap_complete::generate_to(clap_complete::shells::Bashg, clap_app, "esc", out_dir)
            clap_app.gen_completions_to("esc", clap::Shell::Bash, &mut std::io::stdout());
        }

        Command::GenerateZshCompletion => {
            clap_app.gen_completions_to("esc", clap::Shell::Zsh, &mut std::io::stdout());
        }

        Command::GeneratePowershellCompletion => {
            clap_app.gen_completions_to("esc", clap::Shell::PowerShell, &mut std::io::stdout());
        }
    };

    Ok(())
}
