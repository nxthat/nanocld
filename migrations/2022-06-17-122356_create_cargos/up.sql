-- Your SQL goes here
create table "cargoes" (
  "key" VARCHAR NOT NULL UNIQUE PRIMARY KEY,
  "namespace_name" VARCHAR NOT NULL references namespaces("name"),
  "name" VARCHAR NOT NULL,
  "image_name" VARCHAR NOT NULL,
  "binds" TEXT[] NOT NULL,
  "replicas" BIGINT NOT NULL DEFAULT 1 CHECK (replicas >= 0),
  "dns_entry" VARCHAR,
  "domainname" VARCHAR,
  "hostname" VARCHAR,
  "network_mode" VARCHAR,
  "restart_policy" VARCHAR
);
