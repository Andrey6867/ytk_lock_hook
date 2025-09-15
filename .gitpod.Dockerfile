# Файл: .gitpod.Dockerfile

# Начинаем с официального образа Gitpod, в котором уже есть много полезного
FROM gitpod/workspace-full

# Переключаемся на пользователя root для установки системных пакетов
USER root

# Устанавливаем системные зависимости, необходимые для компиляции
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libudev-dev \
    zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

# Возвращаемся к обычному пользователю gitpod
USER gitpod

# Устанавливаем Rust (язык программирования) через rustup (менеджер версий)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/gitpod/.cargo/bin:${PATH}"

# Устанавливаем последнюю стабильную версию Solana Tool Suite
RUN sh -c "$(curl -sSfL https://release.solana.com/stable/install)"
ENV PATH="/home/gitpod/.local/share/solana/install/active_release/bin:${PATH}"

# Устанавливаем Anchor Version Manager (avm), а через него - последнюю версию Anchor
# Это самый правильный способ установки Anchor
RUN cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
RUN avm install latest && avm use latest
