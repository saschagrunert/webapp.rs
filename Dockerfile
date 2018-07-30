FROM scratch

# Copy the build target
COPY target/x86_64-unknown-linux-musl/release/backend /

# Copy the TLS certificates
COPY backend/tls /tls

# Copy the configuration
COPY Config.toml /

# Copy the static files
COPY target/deploy /

# Expose the target port
ARG API_PORT=443
ENV API_PORT ${API_PORT}
EXPOSE ${API_PORT}

# Run the application by default
CMD ["/backend"]
