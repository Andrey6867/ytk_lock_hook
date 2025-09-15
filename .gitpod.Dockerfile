# Начинаем с официального образа Gitpod
FROM gitpod/workspace-full

# Переключаемся на пользователя root для установки системных пакетов
USER root
RUN apt-get update && apt-get install -y \
    build-essential pkg-config libssl-dev libudev-dev zlib1g-dev \
    && rm -rf /var/lib/apt/lists/*

# Возвращаемся к обычному пользователю gitpod
USER gitpod

# Устанавливаем Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/gitpod/.cargo/bin:${PATH}"

# Устанавливаем Solana Tool Suite
RUN sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Устанавливаем Anchor Version Manager (avm) и последнюю версию Anchor
RUN cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
RUN /home/gitpod/.cargo/bin/avm install latest && /home/gitpod/.cargo/bin/avm use latest

# --- НОВЫЙ ВАЖНЫЙ БЛОК ---
# Явно и навсегда прописываем пути в конфигурацию командной строки (.bashrc)
# Это гарантирует, что любой новый терминал будет знать, где искать наши программы.
RUN echo 'export PATH="/home/gitpod/.cargo/bin:$PATH"' >> /home/gitpod/.bashrc
RUN echo 'export PATH="/home/gitpod/.local/share/solana/install/active_release/bin:$PATH"' >> /home/gitpod/.bashrc
RUN echo 'export PATH="/home/gitpod/.avm/bin:$PATH"' >> /home/gitpod/.bashrc
