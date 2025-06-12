FROM ubuntu:latest

RUN adduser -Dh /app gh-webhook

USER gh-webhook

COPY --chown=gh-webhook:gh-webhook ./target/release/github-webhook /app/
WORKDIR /app

ENTRYPOINT [ "/app/github-webhook" ]
EXPOSE ${SERVER_PORT:-8080}
