# syntax=docker/dockerfile:1

########## Builder ##########
# Образ на Debian 12 (bookworm) — та же glibc, что и в distroless/cc-debian12
# в раннере, поэтому бинарь запускается без musl-сборки.
FROM rust:1.98.0-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
ENV RUSTFLAGS="-C target-cpu=broadwell"

WORKDIR /app

# Кэш зависимостей: пустой main.rs с настоящими манифестами.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

# Шаблоны Askama рендерятся на этапе компиляции, поэтому каталог templates/
# нужен только здесь — в рантайм-образ он не попадает.
COPY src ./src
COPY templates ./templates
RUN touch src/main.rs && cargo build --release && strip /app/target/release/webaggregator

########## Runner ##########
# distroless/cc-debian12: glibc + libgcc_s — ровно то, что нужно бинарю
# (ldd: libc, libm, libgcc_s), без shell, пакетного менеджера и лишних утилит.
FROM gcr.io/distroless/cc-debian12 AS runner

WORKDIR /app

COPY --from=builder /app/target/release/webaggregator /app/webaggregator

# static/ читается с диска в рантайме (StaticService: ./static/style.css,
# ./static/favicon.png), поэтому каталог обязан лежать рядом с WORKDIR.
# 65532 — uid/gid встроенного в distroless пользователя nonroot (числа надёжнее имён).
COPY --chown=65532:65532 static ./static

# Корни сертификатов зашиты в бинарь (sqlx tls-rustls-ring-webpki → webpki-roots),
# файл копируем на случай перехода на системный стор CA.
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

# Встроенный в distroless непривилегированный пользователь (uid 65532).
USER nonroot

ENTRYPOINT ["/app/webaggregator"]
