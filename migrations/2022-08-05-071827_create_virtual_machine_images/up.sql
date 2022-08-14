-- Your SQL goes here
CREATE TABLE "virtual_machine_images" (
  "key" VARCHAR NOT NULL UNIQUE PRIMARY KEY,
  "name" VARCHAR NOT NULL,
  "size" BIGINT NOT NULL
)
