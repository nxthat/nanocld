-- Your SQL goes here
CREATE TABLE "virtual_machine_images" (
  "key" VARCHAR NOT NULL UNIQUE PRIMARY KEY,
  "name" VARCHAR NOT NULL,
  "image_path" VARCHAR NOT NULL,
  "size" BIGINT NOT NULL,
  "is_base" BOOLEAN NOT NULL,
  "parent_key" VARCHAR references virtual_machine_images
)
