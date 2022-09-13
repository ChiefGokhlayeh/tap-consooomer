# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.245.2/containers/rust/.devcontainer/base.Dockerfile

# [Choice] Debian OS version (use bullseye on local arm64/Apple Silicon): buster, bullseye
ARG VARIANT="buster"
FROM mcr.microsoft.com/vscode/devcontainers/rust:0-${VARIANT}

RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    curl \
    python3-pip \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN pip3 install --no-cache-dir pre-commit

ARG HADOLINT_VERSION=v2.10.0

RUN curl -sSLo /usr/local/bin/hadolint "https://github.com/hadolint/hadolint/releases/download/${HADOLINT_VERSION}/hadolint-$(uname -s)-$(uname -m)" \
    && chmod +x /usr/local/bin/hadolint
