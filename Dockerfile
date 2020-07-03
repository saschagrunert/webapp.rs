FROM scratch

# Copy the build target
COPY target/x86_64-unknown-linux-musl/release/backend /

# Copy the static files
COPY frontend/favicon.ico /static/favicon.ico
COPY frontend/index.html /static/index.html
COPY frontend/pkg /static/pkg
COPY frontend/css /static/css

# Run the application by default
ENTRYPOINT ["/backend"]
