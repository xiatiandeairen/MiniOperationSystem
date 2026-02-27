FROM rust:latest

# Install nightly and components
RUN rustup install nightly && \
    rustup default nightly && \
    rustup target add x86_64-unknown-none && \
    rustup component add rust-src llvm-tools-preview clippy rustfmt

# Install QEMU and cargo-make
RUN apt-get update && apt-get install -y qemu-system-x86 && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-make

WORKDIR /minios
COPY . .

# Build everything
RUN cargo build --workspace --release \
    -Z "build-std=core,compiler_builtins,alloc" \
    -Z "build-std-features=compiler-builtins-mem"

# Build boot image tool
RUN cd tools/boot-image && cargo build --release --target x86_64-unknown-linux-gnu

# Create boot image
RUN ./tools/boot-image/target/x86_64-unknown-linux-gnu/release/boot-image \
    target/x86_64-unknown-none/release/minios-kernel

CMD ["qemu-system-x86_64", "-drive", "format=raw,file=target/x86_64-unknown-none/release/minios-bios.img", "-nographic", "-m", "256M", "-no-reboot", "-no-shutdown"]
