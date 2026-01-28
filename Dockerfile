FROM debian:bookworm-slim AS bin
ARG TARGETARCH
WORKDIR /out
COPY artifacts/sanitizer-bot-linux-x86_64/sanitizer-bot-linux-x86_64 /tmp/sanitizer-bot-linux-x86_64
COPY artifacts/sanitizer-bot-linux-aarch64/sanitizer-bot-linux-aarch64 /tmp/sanitizer-bot-linux-aarch64
RUN chmod +x /tmp/sanitizer-bot-linux-*
RUN if [ "${TARGETARCH}" = "amd64" ]; then \
	cp /tmp/sanitizer-bot-linux-x86_64 /out/sanitizer-bot; \
	else \
	cp /tmp/sanitizer-bot-linux-aarch64 /out/sanitizer-bot; \
	fi

FROM gcr.io/distroless/cc-debian12
WORKDIR /data
COPY --from=bin /out/sanitizer-bot /app/sanitizer-bot
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
VOLUME ["/data"]
ENTRYPOINT ["/app/sanitizer-bot"]
