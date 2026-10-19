# Kinema media server: the static Rust backend serving the built web UI over HTTPS.
#   docker build -t kinema .                                      the image
#   docker build --target bundle --output type=local,dest=out .   kinema + web/ only (scripts/package-server.sh)

# The web build is the same for every architecture, so it runs on the build machine's own.
FROM --platform=$BUILDPLATFORM node:24-alpine AS web
WORKDIR /src
RUN npm install -g pnpm@12
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml .npmrc ./
COPY apps/web/package.json apps/web/
COPY apps/desktop/package.json apps/desktop/
RUN --mount=type=cache,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile --filter @kinema/web
COPY apps/web apps/web
RUN pnpm --filter @kinema/web build

# Alpine's Rust targets musl and links statically: one binary for every Linux distro.
FROM rust:1-alpine AS backend
RUN apk add --no-cache musl-dev
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates crates
COPY backend backend
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/src/target \
    # COPY keeps source mtimes, which can be older than the cached build; touch so cargo rebuilds our crates.
    find crates backend -type f -exec touch {} + && \
    cargo build --release -p backend && cp target/release/kinema /kinema && \
    mkdir /data

FROM scratch AS bundle
COPY --from=backend /kinema /kinema
COPY --from=web /src/apps/web/dist /web

# Alpine's ffmpeg (musl, same as the backend); VAAPI drivers can be added with apk.
FROM alpine:3.22
RUN apk add --no-cache ffmpeg ca-certificates tzdata
WORKDIR /app
COPY --from=backend /kinema /usr/local/bin/kinema
COPY --from=web /src/apps/web/dist web
COPY --from=backend --chown=1000:1000 /data data
USER 1000:1000
ENV KINEMA_WEB_DIST=/app/web
# kinema.toml (optional) is read from /app; the database, caches and TLS certificate live in /app/data.
VOLUME /app/data
EXPOSE 3000/tcp 48555/udp
CMD ["kinema"]
