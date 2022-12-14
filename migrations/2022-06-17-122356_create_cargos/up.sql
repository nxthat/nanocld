-- Your SQL goes here
create table "cargoes" (
  "key" VARCHAR NOT NULL UNIQUE PRIMARY KEY,
  "namespace_name" VARCHAR NOT NULL references namespaces("name"),
  "name" VARCHAR NOT NULL,
  "config" JSON NOT NULL,
  "replicas" BIGINT NOT NULL DEFAULT 1 CHECK (replicas >= 0),
  "dns_entry" VARCHAR
);
