FROM scratch

# Copy the build target
COPY target/x86_64-unknown-linux-musl/release/backend /

# Copy the TLS certificates
COPY tls /tls

# Copy the configuration
COPY Config.toml /

# Copy the static files
COPY target/deploy /static

# Run the application by default
CMD ["/backend"]
