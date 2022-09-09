-- Your SQL goes here
CREATE TABLE "virtual_machines" (
  "key" VARCHAR NOT NULL PRIMARY KEY,
  "name" VARCHAR NOT NULL,
  "pid_path" VARCHAR NOT NULL,
  "image" VARCHAR NOT NULL references virtual_machine_images,
  "memory" SMALLINT NOT NULL,
  "cpu" SMALLINT NOT NULL,
  "network" VARCHAR NOT NULL,
  "ip_addr" VARCHAR NOT NULL,
  "mac_addr" VARCHAR NOT NULL
);
