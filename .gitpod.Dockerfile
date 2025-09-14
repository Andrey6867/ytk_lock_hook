FROM rust:1.68

RUN apt-get update && apt-get install -y libssl-dev libudev-dev pkg-config zlib1g-dev llvm clang make

RUN sh -c "$(curl -sSfL https://release.solana.com/v1.14.9/install)"
ENV PATH="/root/.local/share/solana/install/active_release/bin:$PATH"

RUN cargo install --git https://github.com/coral-xyz/anchor --tag v0.29.0 anchor-cli --locked
