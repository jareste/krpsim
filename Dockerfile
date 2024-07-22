FROM rust:latest

WORKDIR /usr/src/myapp

COPY . .

CMD ["sh", "-c", "cargo run"]