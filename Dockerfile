ARG PGVER=18

FROM rust:1-bookworm AS build

ARG PGVER

RUN cargo install --locked cargo-pgrx
RUN apt-get update && apt-get install -y bison flex clang

WORKDIR /build/extension

RUN cargo pgrx init "--pg${PGVER}" download

COPY . .

RUN cargo pgrx package --no-default-features --features "pg${PGVER}" -c "$(cargo pgrx info pg-config "pg$PGVER")"
RUN PKGDIR="$(find target/release -maxdepth 5 -type d -name "pgrx-install" -print -quit)";  \
    test -n "$PKGDIR";  \
    mkdir /out; \
    cp -a "$PKGDIR/lib" "$PKGDIR/share" /out/


FROM postgres:${PGVER}-bookworm
ARG PGVER

# Copy packaged artifacts from builder
COPY --from=build /out/lib/postgresql/ /tmp/pg-lib/
COPY --from=build /out/share/postgresql/extension/ /tmp/pg-ext/

# Install into real Postgres dirs discovered via pg_config
RUN set -eux; \
  install -m 755 /tmp/pg-lib/*.so "$(pg_config --pkglibdir)/"; \
  install -m 644 /tmp/pg-ext/*.control "$(pg_config --sharedir)/extension/"; \
  install -m 644 /tmp/pg-ext/*.sql "$(pg_config --sharedir)/extension/"; \
  rm -rf /tmp/pg-lib /tmp/pg-ext
