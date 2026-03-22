FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends curl \
    && curl -fsSL https://github.com/tsl0922/ttyd/releases/latest/download/ttyd.x86_64 \
         -o /usr/local/bin/ttyd \
    && chmod +x /usr/local/bin/ttyd \
    && apt-get remove -y curl \
    && apt-get autoremove -y \
    && rm -rf /var/lib/apt/lists/*

COPY ado /usr/local/bin/ado
COPY config.toml /config.toml

EXPOSE 7681

CMD ["ttyd", "-p", "7681", "ado", "--headless", "--config-file", "/config.toml"]
