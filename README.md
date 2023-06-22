# Envoy Extension in Rust


Here I will build a WASM plugin for Envoy Proxy in Rust using the [proxy wasm sdk](https://github.com/proxy-wasm/proxy-wasm-rust-sdk).

## Development

### Build binary
```sh
rustup target add wasm32-wasi
cargo build --target wasm32-wasi --release
```

### Test 

I use `docker compose` to create a simple cluster: 
- `curl` client as the downstream
- Envoy Proxy
- `httpbin` service as the upstream
- [`jwt_issuer` service](https://github.com/pheely/jwt_issuer) - issues a JWT when requested

![img](img/diagram-dev.png)

**Note:**

1. Make sure all the `service`s share the same `network`. In our case, all three `service`s include the `envoymesh` in their `networks`.

2. Starting from Envoy version 1.20, the child element of `typed_config` is mandatory for `http_filters`. The value should be as follows:

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
$ docker exec -it envoy-extension-in-rust-sleep-1 /bin/sh
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


### Cleanup

Run the following command to stop and remove all containers:

```bash
docker container ls -a --format '{{.Names}}'|grep header | xargs docker rm -f
```

## Docker image

Build image: 

```bash
docker build -t pheely/add_auth_header:0.1 .
```

Publish image to docker hub:

```bash
docker push pheely/add_auth_header:0.1
```

## Apply the filter in Istio

### Sample Application

I will use a ample application to test if the built WASM module works in Istio. The sample app consists two microservices - `Portfolio` and `Pricing` - that are running under the default namespace in the Istio service mesh. 

We also deploy a `Gateway` and two `VirtualService`s to allow traffic outside the mesh to hit both microservices.

![img](img/sample-app.png)

#### Portfolio Service

It accepts an account number and returns the total asset held in the account. It depends on the `Pricing` service for the share prices.

Run the following command to deploy the `portfolio` service and deployment. 
```bash
kubectl apply -f services/portfolio/portfolio.yaml
```

#### Pricing Service

The `pricing` microsservice accepts a ticker symbol and returns the current price of the security. 

To deploy the `pricing` service and deployment, run the following command:
```bash
kubectl apply -f services/pricing/pricing.yaml
```

#### Gateway and VirtualServices

A `Gateway` named `philip-gateway` is applied to the default Istio `ingressgateway`. A `VirtualService` is defined for each microservice to expose it outside the mesh. The hostnames are `portfolio.test.io` and `pricing.test.io` respectively. Run the following command to deploy the gateway and virtual service.

```bash
kubectl apply -f mesh/ingress-gateway.yaml
```

#### Testing the Sample Application before applying the `EnvoyFilter`

We can issue `curl` commands directly from our terminal to test the two microservices. 

```bash
curl -sv -d '{"accountNum": "123456"}' -X POST http://localhost/totalAsset -H "Host: portfolio.test.io"
```

The response should be similar to the following.

```text
{"totalAsset": 4174.00}
```

### Apply `EnvoyFilter`

We will deploy the `add_auth_header.wasm` module as an `EnvoyFilter` on the sidecar of the `portfolio` service. For any outbound request, this filter will call the `jwt_issuer` external service to get a JSON Web Token and add it as the Bearer token of the `Authorization` header.

![img](img/diagram-test.png)

Run the following command to call `pricing` before deploying the filter and notice that there are 6 response headers. <details><summary style="color:Maroon;font-size:16px;"></summary>
```bash
curl -sv -d '{"ticker": "BNS"}' -X POST http://localhost/pricing -H "Host: pricing.test.io"
*   Trying 127.0.0.1:80...
* Connected to localhost (127.0.0.1) port 80 (#0)
> POST /pricing HTTP/1.1
> Host: pricing.test.io
> User-Agent: curl/7.79.1
> Accept: */*
> Content-Length: 17
> Content-Type: application/x-www-form-urlencoded
>
* Mark bundle as not supporting multiuse
< HTTP/1.1 200 OK
< content-type: application/json
< date: Sun, 14 May 2023 16:12:52 GMT
< content-length: 33
< x-envoy-upstream-service-time: 161
< server: istio-envoy
<
* Connection #0 to host localhost left intact
{"ticker": "BNS", "price": 31.26}
```

</details>

Now let's deploy our filters using Istio's `WasmPlugin` that provides a mechanism to extend the functionality of the Istio proxy through Wasm filters. 

Here is the `WasmPlugin` definition for our filter created using SDK. It creates a `WasmPlugin` named `pricing-filter-sdk` that only applies to the `pricing` workload. <details><summary style="color:Maroon;font-size:16px;"></summary>
```yaml
apiVersion: extensions.istio.io/v1alpha1
kind: WasmPlugin
metadata:
  name: pricing-filter-sdk
  namespace: default
spec:
  selector:
    matchLabels:
      app: pricing
  pluginName: add_header
  url: oci://pheely/mywasm:0.2
```

</details>

Run the following command to deploy the wasm plugin: <details><summary style="color:Maroon;font-size:16px;"></summary>

```bash
$ cd code/services/pricing
$ kubectl apply -f pricing-wasm-sdk.yaml
wasmplugin.extensions.istio.io/pricing-filter-sdk created
```

</details>

Check the proxy log to confirm the filter is downloaded successfully: <details><summary style="color:Maroon;font-size:16px;"></summary>
```bash
$ kubectl logs pricing-7cb4c9699f-9cmtl -c istio-proxy
...
2023-05-14T22:31:36.850602Z	info	wasm	fetching image pheely/mywasm from registry index.docker.io with tag 0.2
2023-05-14T16:24:22.979261Z	info	wasm	fetching image pheely/add-header from registry webassemblyhub.io with tag v0.2
```

</details>

Let's hit the `pricing`'s endpoint again and we can see a new header "`x-proxy-wsam-sample: addHeader`" appears in the respones. Now there are total 7 response header. <details><summary style="color:Maroon;font-size:16px;"></summary>
```bash
$ curl -sv -d '{"ticker": "BNS"}' -X POST http://localhost/pricing -H "Host: pricing.test.io"

*   Trying 127.0.0.1:80...
* Connected to localhost (127.0.0.1) port 80 (#0)
> POST /pricing HTTP/1.1
> Host: pricing.test.io
> User-Agent: curl/7.79.1
> Accept: */*
> Content-Length: 17
> Content-Type: application/x-www-form-urlencoded
>
* Mark bundle as not supporting multiuse
< HTTP/1.1 200 OK
< content-type: application/json
< date: Sun, 14 May 2023 16:27:09 GMT
< content-length: 33
< x-envoy-upstream-service-time: 24
< x-proxy-wsam-sample: addHeader
< server: istio-envoy
<
* Connection #0 to host localhost left intact
{"ticker": "BNS", "price": 97.66}
```

</details>

Here is the second `WasmPlugin` -  `pricing-filter-wasme` which is also only applied to the `pricing` workload. <details><summary style="color:Maroon;font-size:16px;"></summary>

```yaml
apiVersion: extensions.istio.io/v1alpha1
kind: WasmPlugin
metadata:
  name: pricing-filter-wasme
  namespace: default
spec:
  selector:
    matchLabels:
      app: pricing          # applied to workload pricing
  pluginName: add_header
  url: oci://webassemblyhub.io/pheely/add-header:v0.2 # url points to the wasm module 
                                                        # on webassemblyhub.io
```

</details>

Run the following command to deploy the `pricing-filter-wasme`: <details><summary style="color:Maroon;font-size:16px;"></summary>

```bash
$ cd code/services/pricing
$ kubectl apply -f pricing-wasm-wasme.yaml
wasmplugin.extensions.istio.io/pricing-filter-wasme created
```

</details>

Check the proxy log to confirm the filter is downloaded successfully: <details><summary style="color:Maroon;font-size:16px;"></summary>
```bash
$ kubectl logs pricing-7cb4c9699f-9cmtl -c istio-proxy
...
2023-05-14T22:31:36.850602Z	info	wasm	fetching image pheely/mywasm from registry index.docker.io with tag 0.2
2023-05-14T16:24:22.979261Z	info	wasm	fetching image pheely/add-header from registry webassemblyhub.io with tag v0.2
```

</details>

Let's hit the `pricing`'s endpoint again and we can see a new header "`hello: world`" appears in the respones. Now there are total 8 response header. <details><summary style="color:Maroon;font-size:16px;"></summary>
```bash
$ curl -sv -d '{"ticker": "BNS"}' -X POST http://localhost/pricing -H "Host: pricing.test.io"

*   Trying 127.0.0.1:80...
* Connected to localhost (127.0.0.1) port 80 (#0)
> POST /pricing HTTP/1.1
> Host: pricing.test.io
> User-Agent: curl/7.79.1
> Accept: */*
> Content-Length: 17
> Content-Type: application/x-www-form-urlencoded
>
* Mark bundle as not supporting multiuse
< HTTP/1.1 200 OK
< content-type: application/json
< date: Sun, 14 May 2023 16:27:09 GMT
< content-length: 33
< x-envoy-upstream-service-time: 19
< hello: world
< x-proxy-wsam-sample: addHeader
< server: istio-envoy
<
* Connection #0 to host localhost left intact
{"ticker": "BNS", "price": 20.98}
```

</details>


<style>
    h1 {
        color: DarkRed;
        text-align: center;
    }
    h2 {
        color: DarkBlue;
    }
    h3 {
        color: DarkGreen;
    }
    h4 {
        color: DarkMagenta;
    }
    strong {
        color: Maroon;
    }
    em {
        color: Maroon;
    }
    img {
        display: block;
        margin-left: auto;
        margin-right: auto
    }
    code {
        color: SlateBlue;
    }
    mark {
        background-color:GoldenRod;
    }
</style>