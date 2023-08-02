# Basic OTLP exporter Example

This example shows basic span and metric usage, and exports to the [OpenTelemetry Collector](https://github.com/open-telemetry/opentelemetry-collector) via OTLP.

## Usage

### `openssl`

```shell
openssl genrsa -out ca-key.pem 2048;
openssl req -new -x509 -nodes -days 1000 -key ca-key.pem -out ca-cert.pem -subj "/C=XX/ST=StateName/L=CityName/O=CompanyName/OU=CompanySectionName/CN=CommonNameOrHostname"
openssl req -newkey rsa:2048 -days 1000 -nodes -keyout server.key -out server-req.pem -subj "/C=XX/ST=StateName/L=CityName/O=CompanyName/OU=CompanySectionName/CN=CommonNameOrHostname"
openssl x509 -req -in server-req.pem -days 1000 -CA ca-cert.pem -CAkey ca-key.pem -set_serial 01 -out server.crt -subj "/C=XX/ST=StateName/L=CityName/O=CompanyName/OU=CompanySectionName/CN=CommonNameOrHostname"
openssl req -newkey rsa:2048 -days 1000 -nodes -keyout client.key -out client-req.pem -subj "/C=XX/ST=StateName/L=CityName/O=CompanyName/OU=CompanySectionName/CN=CommonNameOrHostname"
openssl x509 -req -in client-req.pem -days 1000 -CA ca-cert.pem -CAkey ca-key.pem -set_serial 01 -out client.crt -subj "/C=XX/ST=StateName/L=CityName/O=CompanyName/OU=CompanySectionName/CN=CommonNameOrHostname"

chmod +r server.key
```

### `docker-compose`

By default runs against the `otel/opentelemetry-collector-dev:latest` image, and uses the `tonic`'s
`grpc` example as the transport.

```shell
docker-compose up
or
docker-compose up -d
```

In another terminal run the application `cargo run`

Use the browser to see the trace:

- Jaeger at <http://0.0.0.0:16686>

Tear it down:

```shell
docker-compose down
```

### Manual

If you don't want to use `docker-compose`, you can manually run the `otel/opentelemetry-collector` container
and inspect the logs to see traces being transferred.

```shell
# Run `opentelemetry-collector`
$ docker run  -p4317:4317 otel/opentelemetry-collector:latest

# Report spans/metrics
$ cargo run
```

## View result

You should be able to see something similar below with different time and ID in the same console that docker runs.

### Span

```text
Resource labels:
     -> service.name: STRING(trace-demo)
InstrumentationLibrarySpans #0
InstrumentationLibrary
Span #0
    Trace ID       : 737d9c966e8250475f400776228c0044
    Parent ID      : ade62a071825f2db
    ID             : 7aa9ea5f24e0444c
    Name           : Sub operation...
    Kind           : SPAN_KIND_INTERNAL
    Start time     : 2022-02-24 04:59:57.218995 +0000 UTC
    End time       : 2022-02-24 04:59:57.219022 +0000 UTC
    Status code    : STATUS_CODE_UNSET
    Status message :
Attributes:
     -> lemons: STRING(five)
Events:
SpanEvent #0
     -> Name: Sub span event
     -> Timestamp: 2022-02-24 04:59:57.219012 +0000 UTC
     -> DroppedAttributesCount: 0
ResourceSpans #1
Resource labels:
     -> service.name: STRING(trace-demo)
InstrumentationLibrarySpans #0
InstrumentationLibrary
Span #0
    Trace ID       : 737d9c966e8250475f400776228c0044
    Parent ID      :
    ID             : ade62a071825f2db
    Name           : operation
    Kind           : SPAN_KIND_INTERNAL
    Start time     : 2022-02-24 04:59:57.218877 +0000 UTC
    End time       : 2022-02-24 04:59:57.219043 +0000 UTC
    Status code    : STATUS_CODE_UNSET
    Status message :
Attributes:
     -> ex.com/another: STRING(yes)
Events:
SpanEvent #0
     -> Name: Nice operation!
     -> Timestamp: 2022-02-24 04:59:57.218896 +0000 UTC
     -> DroppedAttributesCount: 0
     -> Attributes:
         -> bogons: INT(100)
```

### Metric

```text
2021-11-19T04:08:36.453Z INFO loggingexporter/logging_exporter.go:56 MetricsExporter {"#metrics": 1}
2021-11-19T04:08:36.454Z DEBUG loggingexporter/logging_exporter.go:66 ResourceMetrics #0
Resource labels:
     -> service.name: STRING(unknown_service)
InstrumentationLibraryMetrics #0
InstrumentationLibrary ex.com/basic
Metric #0
Descriptor:
     -> Name: ex.com.one
     -> Description: A ValueObserver set to 1.0
     -> Unit:
     -> DataType: Gauge
NumberDataPoints #0
Data point attributes:
     -> A: STRING(1)
     -> B: STRING(2)
     -> C: STRING(3)
     -> lemons: INT(10)
StartTimestamp: 2021-11-19 04:07:46.29555 +0000 UTC
Timestamp: 2021-11-19 04:08:36.297279 +0000 UTC
Value: 1.000000
```

GRPC with tls

```bash
env OTLP_TONIC_ENDPOINT=https://localhost:4317 OTLP_TONIC_TLS_PATH=$(pwd) OTLP_TONIC_CA_DOMAIN=testserver.com cargo run
```
