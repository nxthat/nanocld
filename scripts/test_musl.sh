export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_ALL_STATIC=true
export OPENSSL_STATIC=true
export LIBZ_SYS_STATIC=1
cargo build --release
