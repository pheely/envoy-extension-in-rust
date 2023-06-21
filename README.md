# Envoy Extension in Rust

Here I will build a WASM plugin for Envoy Proxy in Rust using the [proxy wasm sdk](https://github.com/proxy-wasm/proxy-wasm-rust-sdk).

## Build

```sh
rustup target add wasm32-wasi
cargo build --target wasm32-wasi --release
```

## Test 

I use `docker compose` to create a simple cluster: 
- `curl` client as the downstream
- Envoy Proxy
- `httpbin` service as the upstream
- [`jwt_issuer` service](https://github.com/pheely/jwt_issuer) - issues a JWT when requested

![img](diagram.png)

Make sure all the `service`s share the same `network`. In our case, all three `service`s include the `envoymesh` in their `networks`.

Starting from Envoy version 1.20, the child element of `typed_config` is mandatory for `http_filters`. The value should be as follows:

```yaml
typed_config:
    "@type": type.googleapis.com/envoy.extensions.filters.http.router.v3.Router
```

Otherwise, Envoy proxy will not start successfully and reports the following error:
```text
Didn't find a registered implementation for 'envoy.filters.http.router' with type URL: ''
```

Run this command to start the cluster.
```bash
docker compose up
```

The following message indicates the proxy connected to the `httpbin` service successfully:

```text
envoy-docker-compose-httpbin-1  | [2023-06-19 04:18:27 +0000] [1] [INFO] Listening at: http://0.0.0.0:8000 (1)
```

Run the following command to talk to the `httpbin` service directly:

```bash
$ docker exec -it envoy-docker-compose-sleep-1 /bin/sh
/ # curl -X GET http://httpbin:8000/headers
{
  "headers": {
    "Accept": "*/*",
    "Host": "httpbin:8000",
    "User-Agent": "curl/8.1.0"
  }
}
```

Run this command to talk to the `httpbin` service through the Envoy Proxy:

```bash
/ # curl -X GET http://envoy:15001/headers
{
  "headers": {
    "Accept": "*/*",
    "Authorization": "Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE2ODczMjUzNDksImV4cCI6MTY4NzMyODk0OSwibmJmIjoxNjg3MzI1MzQ5LCJpc3MiOiJodHRwOi8vMC4wLjAuMDo4MDgwL3Nzby9vYXV0aDIvYXBpIiwic3ViIjoiZmJhOGNlMTAtNjY4OS00MzllLTkzNDQtYzYyY2QxYTEwNDBmIiwiYXVkIjoiZmJhOGNlMTAtNjY4OS00MzllLTkzNDQtYzYyY2QxYTEwNDBmIiwiY3RzIjoiT0FVVEgyX1NUQVRFTEVTU19HUkFOVCIsImF1ZGl0VHJhY2tpbmdJZCI6IjZlOTZmNDM0LTMwODItNGIyMS05MDNmLWI4NDg1MDM5YWZmNCIsInN1Ym5hbWUiOiJmYmE4Y2UxMC02Njg5LTQzOWUtOTM0NC1jNjJjZDFhMTA0MGYiLCJ0b2tlbk5hbWUiOiJhY2Nlc3NfdG9rZW4iLCJ0b2tlbl90eXBlIjoiQmVhcmVyIiwiYXV0aEdyYW50SWQiOiJOcEh1MzNkMC12UTd0ZUI0bDkwcTdrajlmSHMiLCJuYmYiOjE2ODczMjUzNDksImdyYW50X3R5cGUiOiJjbGllbnRfY3JlZGVudGlhbHMiLCJzY29wZSI6WyJzZWN1cml0eS50cy5iZmM4LnRva2VuaXphdGlvbiJdLCJhdXRoX3RpbWUiOjE2ODczMjUzNDksInJlYWxtIjoiL2ludGVybmFsczJzIiwiZXhwIjoxNjg3MzI4OTQ5LCJpYXQiOjE2ODczMjUzNDksImV4cGlyZXNfaW4iOjM2MDAsImp0aSI6Il96NWdDMGhfQjBMQzBPVF85bUM0aVJtMVhRUSJ9.R2BGb8DxT6UzlClsq8JDjn89jNKp8I7vpRYJtlAlZqydjfxxKlRQBfzM7TwHBO25r5U2wMrmQLImll15cDElTe8YQaIimYXbdRHUO0KfWIYcy7JeQ4mkCeCOKIDJmJbJ2t8hTFpwc7l4tolhaCONOgJzbNVUy-mmGYegXQ_UTAjz50ql8W9nUjbQHZsUq3G1WhATv4R5b3NydCQTe0xc75-GoK0xaHO9Fr9bKfUC5QZeYqgWPgRQd5HYRksvLuv8UggVtedHmUH0rTZiX8CcMbQqHjzLjgIgNR7StyRIv8btFcz3KDQvkD5F_JXprPVEBHhwMZvR6-sPELDIzrBvuw",
    "From-Proxy-Wasm": "Hello",
    "Host": "httpbin",
    "User-Agent": "curl/8.1.0",
    "X-Envoy-Expected-Rq-Timeout-Ms": "15000",
    "X-Request-Id": "a8018114-5fad-4e1d-a516-f5f0b886e8e9"
  }
}
```

Notice that four headers are added to the request by the Envoy Proxy & WASM Plugin:

- added by WASM Plugin
  - `Authorization` 
  - `From-Proxy-Wasm`  
- added by Envoy
  - `X-Envoy-Expected-Rq-Timeout-Ms`
  - `X-Request-Id`


## Cleanup

Run the following command to stop and remove all containers:

```bash
docker container ls -a --format '{{.Names}}'|grep header | xargs docker rm -f
```