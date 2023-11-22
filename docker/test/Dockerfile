FROM rust:latest AS builder

WORKDIR /workspace/crate

COPY . .

RUN cargo build

FROM golang:latest

WORKDIR /workspace

ENV DEBIAN_FRONTEND=noninteractive
ENV RUNC_TAG_VERSION=v1.1.7
RUN apt-get update \
    && apt-get install -q -y init systemd vim git cmake make libseccomp-dev\
    gettext-base jq curl tmux g++ lcov libbsd0 libbsd-dev sudo kmod nodejs npm\
    && npm install -g tap\
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace/crate
COPY . .
RUN make install

WORKDIR /workspace

RUN git clone https://github.com/opencontainers/runtime-tools.git

WORKDIR /workspace/runtime-tools

RUN make runtimetest validation-executables