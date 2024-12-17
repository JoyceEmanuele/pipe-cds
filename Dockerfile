# Stage 1: Compilar o projeto Rust
FROM rust:1.79.0 as builder

WORKDIR /app

# Instalar dependências necessárias
RUN apt-get update && \
    apt-get install -y build-essential libssl-dev && \
    cd /tmp && \
    wget https://github.com/Kitware/CMake/releases/download/v3.20.0/cmake-3.20.0.tar.gz && \
    tar -zxvf cmake-3.20.0.tar.gz && \
    cd cmake-3.20.0 && \
    ./bootstrap && \
    make && \
    make install && \
    export CARGO_NET_GIT_FETCH_WITH_CLI=true && \
    apt-get install -y librust-openssl-dev default-libmysqlclient-dev protobuf-compiler

# Copiar o código-fonte restante e compilar
COPY . .
RUN RUSTFLAGS=-Awarnings cargo build --release

# Stage 2: Criar a imagem final
FROM ubuntu:jammy

WORKDIR /app
RUN ls

RUN apt-get update && \
    apt-get install -y ca-certificates libpq5 && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* && \
    cargo install diesel_cli --no-default-features --features postgres   

# Copiar o binário compilado da stage anterior
COPY --from=builder /app/target/release/computed-data-service .
COPY --from=builder /app/configfile.json5 .

# Executar o binário
CMD ["./computed-data-service"]