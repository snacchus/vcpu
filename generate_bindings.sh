cargo install cbindgen
cbindgen --config vcpu-interop/cbindgen.toml vcpu-interop -o target/include/vcpu.h
