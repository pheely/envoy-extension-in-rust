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
    "From-Proxy-Wasm": "Hello",
    "Host": "httpbin",
    "User-Agent": "curl/8.1.0",
    "X-Envoy-Expected-Rq-Timeout-Ms": "15000",
    "X-Request-Id": "ed5e26e8-5c4a-4671-aae6-46b13430b5a7"
  }
}
```

Notice that three headers are created by the Proxy:
- `X-Envoy-Expected-Rq-Timeout-Ms`
- `X-Request-Id`
- `From-Proxy-Wasm`

The first two are from Envoy Proxy out of box. The last one is added by the WASM module.

## Cleanup

Run the following command to stop and remove all containers:

```bash
docker container ls -a --format '{{.Names}}'|grep header | xargs docker rm -f
```