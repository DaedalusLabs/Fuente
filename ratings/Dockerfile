FROM debian:bookworm-slim

RUN set -ex \
    && apt-get update \
    && apt-get install -y curl ca-certificates libssl-dev  --no-install-recommends \
    && rm -rf /var/lib/apt/lists/*

CMD ["./fuente-rating-bot"]
