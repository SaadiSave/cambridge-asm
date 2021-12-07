FROM cimg/rust:1.57

RUN sudo apt update && sudo apt install -y apt-utils

RUN sudo apt install -y gcc-10-aarch64-linux-gnu gcc-10-riscv64-linux-gnu gcc-mingw-w64-x86-64 python3

RUN rustup target add aarch64-unknown-linux-gnu riscv64gc-unknown-linux-gnu

COPY config.toml /home/circleci/.cargo/config.toml

ENV CARGO_HOME=/home/circleci/.cargo

# Install GitHub CLI
RUN curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null && \
    sudo apt update && \
    sudo apt install gh
