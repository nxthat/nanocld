# stage 1 - generate recipe file for dependencies
from rust:1.64.0-alpine3.16 as planner

WORKDIR /app
RUN apk add gcc g++ make
RUN cargo install cargo-chef --locked
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# state 2 - build our dependencies
from rust:1.64.0-alpine3.16 as cacher
WORKDIR /app
COPY --from=planner /usr/local/cargo/bin/cargo-chef /usr/local/cargo/bin/cargo-chef
COPY --from=planner /app/recipe.json ./recipe.json
COPY . .
RUN apk add openssl gcc g++ libpq-dev
RUN cargo chef cook --release --recipe-path recipe.json

# stage 3 - build our project
from rust:1.64.0-alpine3.16 as builder
WORKDIR /app
COPY . .
COPY --from=cacher /app/target /app/target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN apk add libpq-dev
RUN cargo build --release

# stage 4 - create runtime image
from alpine:3.16.2

COPY --from=builder /app/target/release/nanocld /usr/local/bin/nanocld

ENTRYPOINT ["/usr/local/bin/nanocld"]
