-- Your SQL goes here
create table "cargoes" (
  "key" VARCHAR NOT NULL UNIQUE PRIMARY KEY,
  "namespace_name" VARCHAR NOT NULL references namespaces("name"),
  "name" VARCHAR NOT NULL,
  "config" JSON NOT NULL
);
