-- Your SQL goes here
CREATE TYPE "virtual_machine_states" AS ENUM ('running', 'stopped');

CREATE TABLE "virtual_machines" (
  "key" VARCHAR NOT NULL PRIMARY KEY,
  "name" VARCHAR NOT NULL,
  "state" virtual_machine_states NOT NULL,
  "pid_path" VARCHAR NOT NULL,
  "image" VARCHAR NOT NULL references virtual_machine_images,
  "memory" VARCHAR NOT NULL,
  "cpu" SMALLINT NOT NULL,
  "network" VARCHAR NOT NULL,
  "ip_addr" VARCHAR NOT NULL,
  "mac_addr" VARCHAR NOT NULL
);
