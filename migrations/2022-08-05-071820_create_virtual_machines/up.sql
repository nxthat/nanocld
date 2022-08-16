-- Your SQL goes here
CREATE TABLE "virtual_machines" (
  "key" VARCHAR NOT NULL PRIMARY KEY,
  "name" VARCHAR NOT NULL,
  "image" VARCHAR NOT NULL references virtual_machine_images,
  "ip_address" VARCHAR NOT NULL,
  "mac_address" VARCHAR NOT NULL
);
