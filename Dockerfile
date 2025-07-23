# --- Builder Stage ---
# Use the official Rust image corresponding to the target platform.
# Docker Buildx will automatically select the correct architecture (e.g., amd64 or arm64).
FROM rust:1-slim-bullseye as builder

# Set the working directory.
WORKDIR /usr/src/app

# Copy the source code into the container.
COPY . .

# Build the application in release mode.
# Cargo will compile natively for the platform this stage is run on.
RUN cargo build --release

# --- Final Stage ---
# Use a minimal "distroless" base image for a small and secure final image.
# It contains only the application and its runtime dependencies.
FROM gcr.io/distroless/cc-debian11

# Copy the compiled binary from the builder stage.
# The source path is consistent because each platform is built natively in its own builder.
COPY --from=builder /usr/src/app/target/release/whoami /whoami

# Expose the port the application will listen on.
# The PORT environment variable is read by main.rs.
EXPOSE 8080
ENV PORT=8080

# Set the command to run when the container starts.
CMD ["/whoami"]
