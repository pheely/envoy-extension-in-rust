FROM golang:alpine AS builder
RUN apk update && apk add --no-cache git

WORKDIR $GOPATH/src/portfolio
COPY . .

RUN go get -d -v

RUN go build -v -o /usr/local/bin/portfolio 

FROM scratch

COPY --from=builder /usr/local/bin/portfolio /usr/local/bin/portfolio

CMD [ "portfolio" ]