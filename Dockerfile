# Stage 1: Compilar o projeto Rust
FROM rust:1.79.0-slim as builder

WORKDIR /app

# Instalar dependências necessárias para compilação
RUN apt-get update && apt-get install -y \
    build-essential \
    libssl-dev \
    libpq-dev \
    pkg-config \
    protobuf-compiler \
    cmake && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Instalar diesel_cli com suporte a PostgreSQL
RUN cargo install diesel_cli --no-default-features --features postgres

# Copiar o código-fonte e compilar o projeto
COPY . .
RUN RUSTFLAGS=-Awarnings cargo build --release

# Stage 2: Criar a imagem final
FROM ubuntu:jammy

WORKDIR /app

# Instalar apenas as dependências necessárias para execução
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Copiar o binário compilado e arquivos de configuração
COPY --from=builder /app/target/release/computed-data-service .
COPY --from=builder /usr/local/cargo/bin/diesel /usr/local/bin/
COPY --from=builder /app/configfile.json5 .

# Executar o binário
CMD ["./computed-data-service"]
