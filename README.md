Constellation
=============

[![Build Status](https://travis-ci.org/valeriansaliou/constellation.svg?branch=master)](https://travis-ci.org/valeriansaliou/constellation) [![Dependency Status](https://deps.rs/repo/github/valeriansaliou/constellation/status.svg)](https://deps.rs/repo/github/valeriansaliou/constellation) [![Buy Me A Coffee](https://img.shields.io/badge/buy%20me%20a%20coffee-donate-yellow.svg)](https://www.buymeacoffee.com/valeriansaliou)

**Pluggable authoritative DNS server. Entries can be added & removed from an HTTP REST API.**

Constellation is a small authoritative server that lets you manage DNS entries from an HTTP REST API, in a generic way. It can be plugged to your existing infrastructure to manage DNS records for users of your service, eg. to configure outbound email records that cannot be easily wildcarded in a traditional DNS server (DKIM, DMARC, SPF records).

DNS entries are stored in Redis. The DNS database can thus be easily modified and dumped for backup purposes.

_Tested at Rust version: `rustc 1.35.0-nightly (aa99abeb2 2019-04-14)`_

**🇫🇷 Crafted in Angers, France.**

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
* **Geo-DNS** to serve records on a location basis. For instance, serve the IP to your US server for all North America users, and fallback to Europe for the rest. Based on MaxMind GeoLite2 free database, that is automatically updated when necessary.

## How to use it?

### Installation

Constellation is built in Rust. To install it, either download a version from the [Constellation releases](https://github.com/valeriansaliou/constellation/releases) page, use `cargo install` or pull the source code from `master`.

**Install from source:**

If you pulled the source code from Git, you can build it using `cargo`:

```bash
cargo build --release
```

You can find the built binaries in the `./target/release` directory.

_Install `libssl-dev` (ie. OpenSSL headers) before you compile Constellation. SSL dependencies are required for the Geo-DNS database updater._

**Install from Cargo:**

You can install Constellation directly with `cargo install`:

```bash
cargo install constellation-server
```

Ensure that your `$PATH` is properly configured to source the Crates binaries, and then run Constellation using the `constellation` command.

**Install from Docker Hub:**

You might find it convenient to run Constellation via Docker. You can find the pre-built Constellation image on Docker Hub as [valeriansaliou/constellation](https://hub.docker.com/r/valeriansaliou/constellation/).

> Pre-built Docker version may not be the latest version of Constellation available.

First, pull the `valeriansaliou/constellation` image:

```bash
docker pull valeriansaliou/constellation:v1.7.0
```

Then, seed it a configuration file and run it (replace `/path/to/your/constellation/config.cfg` with the path to your configuration file):

```bash
docker run -p 53:53/udp -p 8080:8080 -v /path/to/your/constellation/config.cfg:/etc/constellation.cfg -v /path/to/your/constellation/res/:/var/lib/constellation/ valeriansaliou/constellation:v1.7.0
```

In the configuration file, ensure that:

* `dns.inets` is set to `[0.0.0.0:53]` (this lets Constellation DNS be reached from outside the container)
* `http.inet` is set to `0.0.0.0:8080` (this lets Constellation REST API be reached from outside the container)
* `geo.database_path` is set to `/var/lib/constellation/geo/` (this is where the GeoIP database is stored)

Constellation will be reachable by DNS resolvers from `udp://localhost:53`; while its internal REST API will be reachable from `http://localhost:8080`.

Also, do not forget to initialize the GeoIP database in the `./res/geo/` folder (refer to the part on how to [Initialize GeoIP](https://github.com/valeriansaliou/constellation#initialize-geoip) below).

### Configuration

Use the sample [config.cfg](https://github.com/valeriansaliou/constellation/blob/master/config.cfg) configuration file and adjust it to your own environment.

**Available configuration options are commented below, with allowed values:**

**[server]**

* `log_level` (type: _string_, allowed: `debug`, `info`, `warn`, `error`, default: `error`) — Verbosity of logging, set it to `error` in production

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

**[dns.health]**

* `check_enable` (type: _boolean_, allowed: `true`, `false`, default: `false`) — Whether to perform periodic health checks or not
* `check_interval` (type: _integer_, allowed: seconds, default: `60`) — Interval for which to perform health checks in seconds (from 1 minute to 5 minutes is recommended)

**[[dns.health.http]]**

* `zone` (type: _string_, allowed: any zone root domain, default: no default) — Root domain for zone to be checked (eg. `crisp.email`)
* `name` (type: _string_, allowed: any subdomain on zone, default: no default) — Subdomain for zone to be checked (eg. `inbound.@`, for expanded domain `inbound.crisp.email`)
* `method` (type: _string_, allowed: `HEAD`, `GET`, default: `GET`) — HTTP method to be used by HTTP health probe to perform the check request
* `path` (type: _string_, allowed: HTTP path, default: `/`) — HTTP path to be requested upon check
* `port` (type: _integer_, allowed: TCP ports, default: `443`) — TCP port used for HTTP check
* `timeout` (type: _integer_, allowed: seconds, default: `10`) — Timeout of a single HTTP check attempt
* `retries` (type: _integer_, allowed: numbers, default: `3`) — Maximum number of times to retry a given health check in a row, in the event of a failed health check
* `secure` (type: _boolean_, allowed: `true`, `false`, default: `true`) — Whether to perform health checks over HTTPS or not
* `allow_invalid_certificate` (type: _boolean_, allowed: `true`, `false`, default: `false`) — Whether to allow invalid certificates or not (if health check is performed over HTTPS)
* `expected_status` (type: _integer_, allowed: HTTP status codes, default: `200`) — HTTP response status code to expect
* `expected_body` (type: _string_, allowed: text values, default: empty) — Body contents to expect (sub-string can be contained in response body; only applicable if `method` is set to `GET`)

**[geo]**

* `database_path` (type: _string_, allowed: folder path, default: `./res/geo/`) — Path to the folder containing the GeoIP database
* `database_file` (type: _string_, allowed: file name, default: `GeoLite2-Country.mmdb`) — File name for the GeoIP2 MMDB database in the database folder (either free GeoLite2 or paid GeoIP2; disable `geo.update_enable` if you want to use a custom database)
* `update_enable` (type: _boolean_, allowed: `true`, `false`, default: `true`) — Whether to enable GeoIP database updater or not
* `update_interval` (type: _integer_, allowed: seconds, default: `864000`) — Interval for which to refresh GeoIP database in seconds (1 week or more is recommended)
* `update_url` (type: _string_, allowed: HTTP URL, default: `https://geolite.maxmind.com/download/geoip/database/GeoLite2-Country.tar.gz`) — URL to the compressed GeoIP MMDB file (supported: `tar.gz`), that is downloaded on refresh

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
* `cache_expire_seconds` (type: _integer_, allowed: seconds, default: `60`) — Time in seconds after which a locally-cached record expires and should be refreshed from Redis (this should be kept low)

### Initialize GeoIP

As Constellation does not distribute a GeoIP database in its repository, you will need to fetch it from MaxMind before you run Constellation for the first time (Constellation will refuse to start otherwise).

Execute the provided script:

`./scripts/init_geoip.sh`

_Note that once Constellation started from the GeoIP database you have manually initialized, it will keep the database up-to-date by checking and applying updates automatically in the background. The database initialization is a one-time operation._

### Run Constellation

Constellation can be run as such:

`./constellation -c /path/to/config.cfg`

### Test Constellation

Once running, DNS queries can be made against Constellation over the local network (using the default configuration):

`dig subdomain.crisp.email @::1`

_Note that the `dig` utility can be pointed to a specific server with the `@` modifier, here with IPv6 localhost: `::1`._

## 🛰 HTTP REST API

The Constellation HTTP REST API listens on the configured `http.inet` interface from your `config.cfg` file. You can use it for your management and monitoring needs.

_If you want to play with the API the easy way, an [up-to-date Paw file](https://github.com/valeriansaliou/constellation/tree/master/dev/workspaces) is available with all API routes and example requests. Download the [Paw app for your Mac there](https://paw.cloud/) (Paw a tool developers use to test their APIs)._

### 1. DNS records management

To check, read, insert, modify and delete DNS records, you can use the `zone` API resource.

#### API overview

**Endpoint URL:**

`HTTP http://constellation.local:8080/zone/<zone_name>/record/<record_name>/<record_type>/`

Where:

* `zone_name`: The zone name (ie. base domain), eg. `crisp.email`
* `record_name`: The record name to read or alter (ie. sub-domain or base domain), eg. `inbound.@` for the `inbound.crisp.email` FQDN, or `@` for the `crisp.email` FQDN
* `record_type`: The DNS record type to read or alter for the `record_name`; either: `a`, `aaaa`, `cname`, `mx`, `txt` or `ptr` ([open an issue](https://github.com/valeriansaliou/constellation/issues) if you need support for another record type)

**Request headers:**

* Add an `Authorization` header with a `Basic` authentication where the password is your configured `http.record_token`.

**Geo-DNS regions:**

If you want to serve records to the nearest server using the Geo-DNS feature, you will need to set `regions` via the API, where:

* _Americas_
  * `nnam`: Northern North America
  * `snam`: Southern North America
  * `nsam`: Northern South America
  * `ssam`: Southern South America

* _Europe_
  * `weu`: Western Europe
  * `ceu`: Central Europe
  * `eeu`: Eastern Europe
  * `ru`: Russia

* _Middle East_
  * `me`: Middle East

* _Africa_
  * `naf`: Northern Africa
  * `maf`: Middle Africa
  * `saf`: Southern Africa

* _Asia_
  * `in`: India
  * `seas`: Southeast Asia
  * `neas`: Northeast Asia

* _Oceania_
  * `oc`: Oceania

**Geo-DNS blackhole:**

If you want to return an empty DNS response for blocked countries using the Geo-DNS feature, you will need to set `blackhole` via the API, to a list of blackholed [ISO-3166 Alpha-2 country codes](https://en.wikipedia.org/wiki/List_of_ISO_3166_country_codes) (eg. `FR` for France).

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

{"type":"mx","name":"@","ttl":600,"blackhole": null,"regions": null,"values":["1 inbound.crisp.email","10 inbound-failover.crisp.email"]}
```

##### Write a DNS record (or overwrite existing)

`HTTP PUT http://constellation.local:8080/zone/<zone_name>/record/<record_name>/<record_type>/`

**Example request (standard):**

```http
PUT /zone/crisp.email/record/@/mx HTTP/1.1
Authorization: Basic OlJFUExBQ0VfVEhJU19XSVRIX0FfU0VDUkVUX0tFWQ==
Content-Type: application/json; charset=utf-8

{"values":["1 inbound.crisp.email","10 inbound-failover.crisp.email"],"ttl":600}
```

**Example request (Geo-DNS):**

```http
PUT /zone/crisp.email/record/@/mx HTTP/1.1
Authorization: Basic OlJFUExBQ0VfVEhJU19XSVRIX0FfU0VDUkVUX0tFWQ==
Content-Type: application/json; charset=utf-8

{"regions":{"nnam":["10 inbound-geo.nnam.crisp.email"],"snam":["10 inbound-geo.snam.crisp.email"],"nsam":["10 inbound-geo.nsam.crisp.email"],"ssam":["10 inbound-geo.ssam.crisp.email"],"weu":["10 inbound-geo.weu.crisp.email"],"ceu":["10 inbound-geo.ceu.crisp.email"],"eeu":["10 inbound-geo.eeu.crisp.email"],"ru":["10 inbound-geo.ru.crisp.email"],"me":["10 inbound-geo.me.crisp.email"],"naf":["10 inbound-geo.naf.crisp.email"],"maf":["10 inbound-geo.maf.crisp.email"],"saf":["10 inbound-geo.saf.crisp.email"],"in":["10 inbound-geo.in.crisp.email"],"seas":["10 inbound-geo.seas.crisp.email"],"neas":["10 inbound-geo.neas.crisp.email"],"oc":["10 inbound-geo.oc.crisp.email"]},"values":["1 inbound.crisp.email","10 inbound-failover.crisp.email"],"ttl":600}
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

### 2. Server usage metrics retrieval

To obtain server usage metrics (eg. which countries DNS requests come), you can use the `metrics` API resource.

#### API overview

**Endpoint URL:**

`HTTP http://constellation.local:8080/zone/<zone_name>/metrics/<metrics_timespan>/<metrics_category>/<metrics_type>`

Where:

* `zone_name`: The zone name (ie. base domain), eg. `crisp.email`
* `metrics_timespan`: The timespan over which metrics should be returned (either: `1m`, `5m` or `15m`), which stands for: _metrics for the last 'n-th' minutes_
* `metrics_category`: The metrics category (either: `query` or `answer`)
* `metrics_type`: The metrics type in category (either: `types` or `origins` if category is `query`, or `codes` if category is `answer`)

**Request headers:**

* Add an `Authorization` header with a `Basic` authentication where the password is your configured `http.record_token`.

#### API routes

##### Get metrics

`HTTP GET http://constellation.local:8080/zone/<zone_name>/metrics/<metrics_timespan>/<metrics_category>/<metrics_type>/`

**Example request:**

```http
GET /zone/crisp.email/metrics/5m/query/origins HTTP/1.1
Authorization: Basic OlJFUExBQ0VfVEhJU19XSVRIX0FfU0VDUkVUX0tFWQ==
```

**Example response:**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{"fr":1203,"us":899,"lv":23,"gb":10,"other":2}
```

## :fire: Report A Vulnerability

If you find a vulnerability in Constellation, you are more than welcome to report it directly to [@valeriansaliou](https://github.com/valeriansaliou) by sending an encrypted email to [valerian@valeriansaliou.name](mailto:valerian@valeriansaliou.name). Do not report vulnerabilities in public GitHub issues, as they may be exploited by malicious people to target production servers running an unpatched Constellation instance.

**:warning: You must encrypt your email using [@valeriansaliou](https://github.com/valeriansaliou) GPG public key: [:key:valeriansaliou.gpg.pub.asc](https://valeriansaliou.name/files/keys/valeriansaliou.gpg.pub.asc).**

**:gift: Based on the severity of the vulnerability, I may offer a $200 (US) bounty to whomever reported it.**
