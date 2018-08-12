Constellation
=============

[![Build Status](https://travis-ci.org/valeriansaliou/constellation.svg?branch=master)](https://travis-ci.org/valeriansaliou/constellation) [![Dependency Status](https://deps.rs/repo/github/valeriansaliou/constellation/status.svg)](https://deps.rs/repo/github/valeriansaliou/constellation)

**Pluggable authoritative DNS server. Entries can be added & removed from an HTTP REST API.**

Constellation is a small authoritative server that lets you manage DNS entries from an HTTP REST API, in a generic way. It can be plugged to your existing infrastructure to manage DNS records for users of your service, eg. to configure outbound email records that cannot be easily wildcarded in a traditional DNS server (DKIM, DMARC, SPF records).

DNS entries are stored in Redis. The DNS database can thus be easily modified and dumped for backup purposes.

![Constellation](https://valeriansaliou.github.io/constellation/images/constellation.jpg)

## Who uses it?

<table>
<tr>
<td align="center"><a href="https://crisp.chat/"><img src="https://valeriansaliou.github.io/constellation/images/crisp.png" height="64" /></a></td>
</tr>
<tr>
<td align="center">Crisp</td>
</tr>
</table>

_👋 You use Constellation and you want to be listed there? [Contact me](https://valeriansaliou.name/)._

## Features

* **Pluggable authoritative DNS server**, comes handy if you need to generate eg. email sub-domains for your users (with DKIM, DMARC and SPF records).
* **HTTP REST API** to check, read, insert, modify and delete DNS records on the fly.
* **Persistence layer** in Redis. This means you can run multiple Constellations hitting against the same database on the network. You can even shard Redis if you need fault tolerance on the DNS data store.

## How to use it?

### Installation

Constellation is built in Rust. To install it, either download a version from the [Constellation releases](https://github.com/valeriansaliou/constellation/releases) page, use `cargo install` or pull the source code from `master`.

**Install from source:**

If you pulled the source code from Git, you can build it using `cargo`:

```bash
cargo build --release
```

You can find the built binaries in the `./target/release` directory.

**Install from Cargo:**

You can install Constellation directly with `cargo install`:

```bash
cargo install constellation-server
```

Ensure that your `$PATH` is properly configured to source the Crates binaries, and then run Constellation using the `constellation` command.

### Configuration

Use the sample [config.cfg](https://github.com/valeriansaliou/constellation/blob/master/config.cfg) configuration file and adjust it to your own environment.

**Available configuration options are commented below, with allowed values:**

**[server]**

* `log_level` (type: _string_, allowed: `debug`, `info`, `warn`, `error`, default: `warn`) — Verbosity of logging, set it to `error` in production

**[dns]**

* `inets` (type: _array[string]_, allowed: IPs + ports, default: `[0.0.0.0:53, [::]:53]`) — Hosts and UDP/TCP ports the DNS server should listen on
* `tcp_timeout` (type: _integer_, allowed: seconds, default: `2`) — Timeout of DNS over TCP connections
* `nameservers` (type: _array[string]_, allowed: domain names, default: no default) — Name server domains for all served domains
* `soa_master` (type: _string_, allowed: domain names, default: no default) — SOA master domain for all zones served by this name server (name of primary NS server)
* `soa_responsible` (type: _string_, allowed: email addresses as domain names, default: no default) — SOA responsible email for all zones served by this name server
* `soa_refresh` (type: _integer_, allowed: seconds, default: `10000`) — SOA record refresh value
* `soa_retry` (type: _integer_, allowed: seconds, default: `2400`) — SOA record retry value
* `soa_expire` (type: _integer_, allowed: seconds, default: `604800`) — SOA record expire value
* `soa_ttl` (type: _integer_, allowed: seconds, default: `3600`) — SOA record TTL value
* `record_ttl` (type: _integer_, allowed: seconds, default: `3600`) — DNS records TTL value

**[[dns.zone.'{name}']]**

> Specify your zone name eg. as: `[[dns.zone.'crisp.email']]` for zone base: `crisp.email`.

**[http]**

* `inet` (type: _string_, allowed: IPv4 / IPv6 + port, default: `[::1]:8080`) — Host and TCP port the HTTP API server should listen on
* `workers` (type: _integer_, allowed: any number, default: `2`) — Number of workers for the HTTP API server to run on
* `record_token` (type: _string_, allowed: secret token, default: no default) — Record secret token for management API access (ie. secret password)

**[redis]**

* `host` (type: _string_, allowed: hostname, IPv4, IPv6, default: `localhost`) — Target Redis host
* `port` (type: _integer_, allowed: TCP port, default: `6379`) — Target Redis TCP port
* `password` (type: _string_, allowed: password values, default: none) — Redis password (if no password, dont set this key)
* `database` (type: _integer_, allowed: `0` to `255`, default: `0`) — Target Redis database
* `pool_size` (type: _integer_, allowed: `0` to `(2^32)-1`, default: `8`) — Redis connection pool size
* `max_lifetime_seconds` (type: _integer_, allowed: seconds, default: `20`) — Maximum lifetime of a connection to Redis (you want it below 5 minutes, as this affects the reconnect delay to Redis if a connection breaks)
* `idle_timeout_seconds` (type: _integer_, allowed: seconds, default: `600`) — Timeout of idle/dead pool connections to Redis
* `connection_timeout_seconds` (type: _integer_, allowed: seconds, default: `5`) — Timeout in seconds to consider Redis dead and reject DNS and HTTP API queries

### Run Constellation

Constellation can be run as such:

`./constellation -c /path/to/config.cfg`

### DNS records management (HTTP REST API)

To check, read, insert, modify and delete DNS records, you need to use the Constellation HTTP REST API, that listens on the configured `http.inet` interface from your `config.cfg` file.

#### API overview

**Endpoint URL:**

`HTTP http://constellation.local:8080/zone/<zone_name>/record/<record_name>/<record_type>/`

Where:

* `zone_name`: The zone name (ie. base domain), eg. `crisp.email`
* `record_name`: The record name to read or alter (ie. sub-domain or base domain), eg. `inbound.@` for the `inbound.crisp.email` FQDN, or `@` for the `crisp.email` FQDN
* `record_type`: The DNS record type to read or alter for the `record_name`; either: `a`, `aaaa`, `cname`, `mx`, `txt` ([open an issue](https://github.com/valeriansaliou/constellation/issues) if you need support for another record type)

**Request headers:**

* Add an `Authorization` header with a `Basic` authentication where the password is your configured `http.record_token`.

#### API regions

If you want to serve records to the nearest server using the Geo-DNS feature, you will need to set `regions` via the API, where:

* `EU`: Europe
* `NAM`: North America
* `SAM`: South America
* `OC`: Oceania
* `AS`: Asia
* `AF`: Africa

#### API routes

##### Check if a DNS record exists

`HTTP HEAD http://constellation.local:8080/zone/<zone_name>/record/<record_name>/<record_type>/`

**Example request:**

```http
HEAD /zone/crisp.email/record/@/mx HTTP/1.1
Authorization: Basic OlJFUExBQ0VfVEhJU19XSVRIX0FfU0VDUkVUX0tFWQ==
```

**Example response:**

```http
HTTP/1.1 200 OK
```

##### Get a DNS record

`HTTP GET http://constellation.local:8080/zone/<zone_name>/record/<record_name>/<record_type>/`

**Example request:**

```http
GET /zone/crisp.email/record/@/mx HTTP/1.1
Authorization: Basic OlJFUExBQ0VfVEhJU19XSVRIX0FfU0VDUkVUX0tFWQ==
```

**Example response:**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{"type":"mx","name":"@","ttl":600,"regions": null,"values":["1 inbound.crisp.email","10 inbound-failover.crisp.email"]}
```

##### Write a DNS record (or overwrite existing)

`HTTP PUT http://constellation.local:8080/zone/<zone_name>/record/<record_name>/<record_type>/`

**Example request:**

```http
PUT /zone/crisp.email/record/@/mx HTTP/1.1
Authorization: Basic OlJFUExBQ0VfVEhJU19XSVRIX0FfU0VDUkVUX0tFWQ==
Content-Type: application/json; charset=utf-8

{"values":["1 inbound.crisp.email","10 inbound-failover.crisp.email"],"regions":{"EU":["10 inbound-failover.europe.crisp.email"],"NAM":["10 inbound-failover.americas.crisp.email"],"SAM":["10 inbound-failover.americas.crisp.email"],"OC":["10 inbound-failover.asia.crisp.email"],"AS":["10 inbound-failover.asia.crisp.email"],"AF":["10 inbound-failover.europe.crisp.email"]},"ttl":600}
```

**Example response:**

```http
HTTP/1.1 200 OK
```

##### Delete a DNS record

`HTTP DELETE http://constellation.local:8080/zone/<zone_name>/record/<record_name>/<record_type>/`

**Example request:**

```http
DELETE /zone/crisp.email/record/@/mx HTTP/1.1
Authorization: Basic OlJFUExBQ0VfVEhJU19XSVRIX0FfU0VDUkVUX0tFWQ==
```

**Example response:**

```http
HTTP/1.1 200 OK
```

## :fire: Report A Vulnerability

If you find a vulnerability in Constellation, you are more than welcome to report it directly to [@valeriansaliou](https://github.com/valeriansaliou) by sending an encrypted email to [valerian@valeriansaliou.name](mailto:valerian@valeriansaliou.name). Do not report vulnerabilities in public GitHub issues, as they may be exploited by malicious people to target production servers running an unpatched Constellation instance.

**:warning: You must encrypt your email using [@valeriansaliou](https://github.com/valeriansaliou) GPG public key: [:key:valeriansaliou.gpg.pub.asc](https://valeriansaliou.name/files/keys/valeriansaliou.gpg.pub.asc).**

**:gift: Based on the severity of the vulnerability, I may offer a $200 (US) bounty to whomever reported it.**
