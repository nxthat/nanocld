// @generated automatically by Diesel CLI.

pub mod sql_types {
  #[derive(diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "node_modes"))]
  pub struct NodeModes;

  #[derive(diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "proxy_template_modes"))]
  pub struct ProxyTemplateModes;

  #[derive(diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "ssh_auth_modes"))]
  pub struct SshAuthModes;
}

diesel::table! {
    cargo_environnements (key) {
        key -> Varchar,
        cargo_key -> Varchar,
        name -> Varchar,
        value -> Varchar,
    }
}

diesel::table! {
    cargo_instances (key) {
        key -> Varchar,
        cargo_key -> Varchar,
        cluster_key -> Varchar,
        network_key -> Varchar,
    }
}

diesel::table! {
    cargoes (key) {
        key -> Varchar,
        namespace_name -> Varchar,
        name -> Varchar,
        config -> Jsonb,
        replicas -> Int8,
        dns_entry -> Nullable<Varchar>,
    }
}

diesel::table! {
    cluster_networks (key) {
        key -> Varchar,
        name -> Varchar,
        namespace -> Varchar,
        docker_network_id -> Varchar,
        default_gateway -> Varchar,
        cluster_key -> Varchar,
    }
}

diesel::table! {
    cluster_variables (key) {
        key -> Varchar,
        cluster_key -> Varchar,
        name -> Varchar,
        value -> Varchar,
    }
}

diesel::table! {
    clusters (key) {
        key -> Varchar,
        name -> Varchar,
        namespace -> Varchar,
        proxy_templates -> Array<Text>,
    }
}

diesel::table! {
    namespaces (name) {
        name -> Varchar,
    }
}

diesel::table! {
    nginx_logs (key) {
        key -> Uuid,
        date_gmt -> Timestamptz,
        uri -> Varchar,
        host -> Varchar,
        remote_addr -> Varchar,
        realip_remote_addr -> Varchar,
        server_protocol -> Varchar,
        request_method -> Varchar,
        content_length -> Int8,
        status -> Int8,
        request_time -> Float8,
        body_bytes_sent -> Int8,
        proxy_host -> Nullable<Varchar>,
        upstream_addr -> Nullable<Varchar>,
        query_string -> Nullable<Varchar>,
        request_body -> Nullable<Varchar>,
        content_type -> Nullable<Varchar>,
        http_user_agent -> Nullable<Varchar>,
        http_referrer -> Nullable<Varchar>,
        http_accept_language -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::NodeModes;
    use super::sql_types::SshAuthModes;

    nodes (name) {
        name -> Varchar,
        mode -> NodeModes,
        ip_address -> Varchar,
        ssh_auth_mode -> SshAuthModes,
        ssh_user -> Varchar,
        ssh_credential -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ProxyTemplateModes;

    proxy_templates (name) {
        name -> Varchar,
        mode -> ProxyTemplateModes,
        content -> Text,
    }
}

diesel::joinable!(cargoes -> namespaces (namespace_name));
diesel::joinable!(cargo_instances -> cargoes (cargo_key));
diesel::joinable!(cargo_instances -> cluster_networks (network_key));
diesel::joinable!(cargo_instances -> clusters (cluster_key));
diesel::joinable!(cluster_networks -> clusters (cluster_key));

diesel::allow_tables_to_appear_in_same_query!(
  cargo_environnements,
  cargo_instances,
  cargoes,
  cluster_networks,
  cluster_variables,
  clusters,
  namespaces,
  nginx_logs,
  nodes,
  proxy_templates,
);
