-- Your SQL goes here
CREATE TABLE "cargo_instances" (
  "key" VARCHAR NOT NULL UNIQUE PRIMARY KEY,
  "cargo_key" VARCHAR NOT NULL references cargoes("key"),
  "cluster_key" VARCHAR NOT NULL references clusters("key"),
  "network_key" VARCHAR NOT NULL references cluster_networks("key")
);
