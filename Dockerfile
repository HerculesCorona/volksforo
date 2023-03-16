# syntax=docker/dockerfile:1

FROM debian:stable-slim
WORKDIR /app
COPY target/debug/volksforo /app/volksforo
COPY .env .env
CMD ["/app/volksforo"]
EXPOSE 8080
