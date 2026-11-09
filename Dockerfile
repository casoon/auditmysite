# auditmysite container image (#927).
#
# Contains the released binary (downloaded from the GitHub release and
# verified against the published .sha256, never rebuilt here) plus Debian's
# Chromium. trixie, not bookworm: the x86_64 release binary is linked against
# the glibc of the ubuntu-latest runner, which bookworm's glibc is too old for.
#
#   docker build --build-arg VERSION=v2.0.2 -t auditmysite .
#   docker run --rm auditmysite https://example.com -f json
ARG DEBIAN_TAG=trixie-slim

FROM debian:${DEBIAN_TAG} AS fetch
ARG VERSION
ARG TARGETARCH
ARG REPO=casoon/auditmysite
RUN test -n "${VERSION}" || { echo "build-arg VERSION (e.g. v2.0.2) is required" >&2; exit 1; }
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*
WORKDIR /dl
RUN case "${TARGETARCH}" in \
      amd64) target=x86_64-unknown-linux-gnu ;; \
      arm64) target=aarch64-unknown-linux-gnu ;; \
      *) echo "unsupported architecture: ${TARGETARCH}" >&2; exit 1 ;; \
    esac \
 && archive="auditmysite-${VERSION}-${target}.tar.gz" \
 && url="https://github.com/${REPO}/releases/download/${VERSION}/${archive}" \
 && curl -fsSL --retry 3 -o "${archive}" "${url}" \
 && curl -fsSL --retry 3 -o "${archive}.sha256" "${url}.sha256" \
 && sha256sum -c "${archive}.sha256" \
 && tar -xzf "${archive}" auditmysite

FROM debian:${DEBIAN_TAG}
ARG VERSION
LABEL org.opencontainers.image.title="auditmysite" \
      org.opencontainers.image.description="WCAG 2.2 AA accessibility checker with bundled Chromium" \
      org.opencontainers.image.source="https://github.com/casoon/auditmysite" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.version="${VERSION}"
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates chromium fonts-liberation fonts-noto-color-emoji \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --create-home --uid 10001 auditmysite
COPY --from=fetch /dl/auditmysite /usr/local/bin/auditmysite
USER auditmysite
WORKDIR /work
ENV AUDITMYSITE_BROWSER=/usr/bin/chromium
# Chromium's sandbox needs user namespaces that a default container does not
# grant; the container is the isolation boundary.
ENTRYPOINT ["auditmysite", "--no-sandbox"]
