FROM golang:alpine AS builder
RUN apk update && apk add --no-cache git

WORKDIR $GOPATH/src/pricing
COPY . .

RUN go get -d -v

RUN go build -v -o /usr/local/bin/pricing 

FROM scratch

COPY --from=builder /usr/local/bin/pricing /usr/local/bin/pricing

CMD [ "pricing" ]