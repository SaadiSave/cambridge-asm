FROM rust:1.82-bookworm

RUN apt update && apt install -y gcc-aarch64-linux-gnu gcc-riscv64-linux-gnu gcc-mingw-w64-x86-64 python3 zip

RUN rustup target add aarch64-unknown-linux-gnu riscv64gc-unknown-linux-gnu x86_64-pc-windows-gnu

RUN rustup component add rustfmt clippy

RUN cargo install cargo-hack cargo-nextest

# Install GitHub CLI
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null && \
    apt update && \
    apt install gh
