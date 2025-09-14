# Шаг 1: Начинаем со старого, стабильного Rust
FROM rust:1.65.0

# Шаг 2: Устанавливаем базовые зависимости
RUN apt-get update && apt-get install -y libssl-dev libudev-dev pkg-config zlib1g-dev llvm clang make

# Шаг 3: Устанавливаем стабильную версию Solana
RUN sh -c "$(curl -sSfL https://release.solana.com/v1.14.17/install)"
ENV PATH="/root/.local/share/solana/install/active_release/bin:$PATH"

# Шаг 4: Устанавливаем стабильную версию Anchor
RUN cargo install anchor-cli --git https://github.com/coral-xyz/anchor --tag v0.26.0 --locked --force
