Constellation
=============

[![Test and Build](https://github.com/valeriansaliou/constellation/workflows/Test%20and%20Build/badge.svg?branch=master)](https://github.com/valeriansaliou/constellation/actions?query=workflow%3A%22Test+and+Build%22) [![Build and Release](https://github.com/valeriansaliou/constellation/workflows/Build%20and%20Release/badge.svg)](https://github.com/valeriansaliou/constellation/actions?query=workflow%3A%22Build+and+Release%22) [![dependency status](https://deps.rs/repo/github/valeriansaliou/constellation/status.svg)](https://deps.rs/repo/github/valeriansaliou/constellation) [![Buy Me A Coffee](https://img.shields.io/badge/buy%20me%20a%20coffee-donate-yellow.svg)](https://www.buymeacoffee.com/valeriansaliou)

**Pluggable authoritative DNS server. Entries can be added & removed from an HTTP REST API.**

Constellation is a small authoritative server that lets you manage DNS entries from an HTTP REST API, in a generic way. It can be plugged to your existing infrastructure to manage DNS records for users of your service, eg. to configure outbound email records that cannot be easily wildcarded in a traditional DNS server (DKIM, DMARC, SPF records).

DNS entries are stored in Redis. The DNS database can thus be easily modified and dumped for backup purposes.

_Tested at Rust version: `rustc 1.79.0 (129f3b996 2024-06-10)`_

**🇫🇷 Crafted in Angers, France.**

![Constellation](https://valeriansaliou.github.io/constellation/images/constellation.jpg)

## Who uses it?

<table>
<tr>
<td align="center"><a href="https://crisp.chat/"><img src="https://valeriansaliou.github.io/constellation/images/crisp.png" width="64" /></a></td>
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
* **CNAME flattening** to save resolution round-trips for the users you serve. This can be enabled on a per record basis.

## Supported Record Types

Constellation supports all of the most widely used DNS record types:

| Type    | Supported? | CNAME flattening? |
|---------|------------|-------------------|
| `A`     | ✅         | ✅                |
| `AAAA`  | ✅         | ✅                |
| `CNAME` | ✅         | ❌                 |
| `MX`    | ✅         | ✅                |
| `TXT`   | ✅         | ✅                |
| `CAA`   | ✅         | ✅                |
| `PTR`   | ✅         | ❌                 |

👉 _If you would like me to add support for a record type that is not listed here, please [open an issue](https://github.com/valeriansaliou/constellation/issues)._

## How to use it?

### Installation

Constellation is built in Rust. To install it, either download a version from the [Constellation releases](https://github.com/valeriansaliou/constellation/releases) page, use `cargo install` or pull the source code from `master`.

👉 _Each release binary comes with an `.asc` signature file, which can be verified using [@valeriansaliou](https://github.com/valeriansaliou) GPG public key: [:key:valeriansaliou.gpg.pub.asc](https://valeriansaliou.name/files/keys/valeriansaliou.gpg.pub.asc)._

**Install from packages:**

Constellation provides [pre-built packages](https://packagecloud.io/valeriansaliou/constellation) for Debian-based systems (Debian, Ubuntu, etc.).

**Important: Constellation only provides 64 bits packages targeting Debian 10, 11 & 12 for now (codenames: `buster`, `bullseye` & `bookworm`). You will still be able to use them on other Debian versions, as well as Ubuntu.**

First, add the Constellation APT repository (eg. for Debian `bookworm`):

```bash
echo "deb [signed-by=/usr/share/keyrings/valeriansaliou_constellation.gpg] https://packagecloud.io/valeriansaliou/constellation/debian/ bookworm main" > /etc/apt/sources.list.d/valeriansaliou_constellation.list
```

```bash
curl -fsSL https://packagecloud.io/valeriansaliou/constellation/gpgkey | gpg --dearmor -o /usr/share/keyrings/valeriansaliou_constellation.gpg
```

```bash
apt-get update
```

Then, install the Constellation package:

```bash
apt-get install constellation
```

Then, edit the pre-filled Constellation configuration file:

```bash
nano /etc/constellation.cfg
```

Finally, restart Constellation:

```
service constellation restart
```

**Install from source:**

If you pulled the source code from Git, you can build it using `cargo`:

```bash
cargo build --release
```

You can find the built binaries in the `./target/release` directory.

_Install `libssl-dev` (ie. OpenSSL headers) before you compile Constellation. SSL dependencies are required for the Geo-DNS database updater and the DNS health check system (HTTPS prober)._

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
docker pull valeriansaliou/constellation:v1.15.0
```

Then, seed it a configuration file and run it (replace `/path/to/your/constellation/config.cfg` with the path to your configuration file):

```bash
docker run -p 53:53/udp -p 8080:8080 -v /path/to/your/constellation/config.cfg:/etc/constellation.cfg -v /path/to/your/constellation/res/:/var/lib/constellation/ valeriansaliou/constellation:v1.15.0
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
* `identifier` (type: _string_, allowed: text values, default: `constellation/0`) — Identifier of this Constellation server in the pool of replicas (used for identification and notification purposes)

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

> Specify your zone name eg. as: `[[dns.zone.'relay.crisp.chat']]` for zone base: `relay.crisp.chat`.

**[dns.flatten]**

* `resolvers` (type: _array[string]_, allowed: hostname, IPv4, IPv6, default: no default) — DNS resolvers that should be used when flattening a CNAME record

**[dns.health]**

* `check_enable` (type: _boolean_, allowed: `true`, `false`, default: `false`) — Whether to perform periodic health checks or not
* `check_interval` (type: _integer_, allowed: seconds, default: `60`) — Interval for which to perform health checks in seconds (from 1 minute to 5 minutes is recommended)

**[dns.health.notify]**

* `slack_hook_url` (type: _string_, allowed: URL, default: no default) — Slack hook URL for notifications (ie. `https://hooks.slack.com/[..]`)

**[[dns.health.http]]**

* `zone` (type: _string_, allowed: any zone root domain, default: no default) — Root domain for zone to be checked (eg. `relay.crisp.chat`)
* `name` (type: _string_, allowed: any subdomain on zone, default: no default) — Subdomain for zone to be checked (eg. `client.@`, for expanded domain `client.relay.crisp.chat`)
* `method` (type: _string_, allowed: `HEAD`, `GET`, default: `GET`) — HTTP method to be used by HTTP health probe to perform the check request
* `host` (type: _string_, allowed: HTTP virtual hosts, default: empty) — HTTP virtual host to be requested upon check (if not set, it is generated from `zone` and `name`)
* `path` (type: _string_, allowed: HTTP paths, default: `/`) — HTTP path to be requested upon check
* `port` (type: _integer_, allowed: TCP ports, default: `443`) — TCP port used for HTTP check (port value will likely be `80` if HTTP is used)
* `secure` (type: _boolean_, allowed: `true`, `false`, default: `true`) — Whether to perform health checks over secure HTTPS or plain HTTP
* `timeout` (type: _integer_, allowed: seconds, default: `10`) — Timeout of a single HTTP check attempt in seconds
* `max_attempts` (type: _integer_, allowed: numbers, default: `3`) — Maximum number of times to attempt a given health check in a row, in the event of a failed health check (ie. an health check that neither matches expected status and expected body)
* `expected_status` (type: _array[integer]_, allowed: HTTP status codes, default: `200`) — List of HTTP status codes to expect
* `expected_body` (type: _array[string]_, allowed: text values, default: empty) — List of body contents to expect (sub-string can be contained in response body; only applicable if `method` is set to `GET`)

**[geo]**

* `database_path` (type: _string_, allowed: folder path, default: `./res/geo/`) — Path to the folder containing the GeoIP database
* `database_file` (type: _string_, allowed: file name, default: `GeoLite2-Country.mmdb`) — File name for the GeoIP2 MMDB database in the database folder (either free GeoLite2 or paid GeoIP2; enable `geo.update_enable` if you want to automatically update this file from a remote download server)
* `update_enable` (type: _boolean_, allowed: `true`, `false`, default: `false`) — Whether to enable GeoIP database updater or not
* `update_interval` (type: _integer_, allowed: seconds, default: `864000`) — Interval for which to refresh GeoIP database in seconds (1 week or more is recommended)
* `update_url` (type: _string_, allowed: HTTP URL, default: empty) — URL to the compressed GeoIP MMDB file (supported: `tar.gz`), that is downloaded on refresh (a value is required if `geo.update_enable` is enabled)

**[http]**

* `inet` (type: _string_, allowed: IPv4 / IPv6 + port, default: `[::1]:8080`) — Host and TCP port the HTTP API server should listen on
* `workers` (type: _integer_, allowed: any number, default: `2`) — Number of workers for the HTTP API server to run on
* `record_token` (type: _string_, allowed: secret token, default: no default) — Record secret token for management API access (ie. secret password)

**[redis]**

* `database` (type: _integer_, allowed: `0` to `255`, default: `0`) — Target Redis database
* `pool_size` (type: _integer_, allowed: `0` to `(2^32)-1`, default: `8`) — Redis connection pool size
* `max_lifetime_seconds` (type: _integer_, allowed: seconds, default: `20`) — Maximum lifetime of a connection to Redis (you want it below 5 minutes, as this affects the reconnect delay to Redis if a connection breaks)
* `idle_timeout_seconds` (type: _integer_, allowed: seconds, default: `600`) — Timeout of idle/dead pool connections to Redis
* `connection_timeout_seconds` (type: _integer_, allowed: seconds, default: `2`) — Timeout in seconds to consider Redis dead and reject DNS and HTTP API queries
* `delinquency_seconds` (type: _integer_, allowed: seconds, default: `10`) — Time in seconds to mark a failing Redis connection as delinquent, meaning it will not be retried over and over again during the failure period. It should be some factor of the connection timeout, a factor of 3 is recommended.
* `cache_refresh_seconds` (type: _integer_, allowed: seconds, default: `60`) — Time in seconds after which a locally-cached record is refreshed from Redis (this should be kept low)
* `cache_expire_seconds` (type: _integer_, allowed: seconds, default: `600`) — Time in seconds after which a locally-cached record expires and should be refreshed from Redis (this should be kept low)

**[redis.master]**

* `host` (type: _string_, allowed: hostname, IPv4, IPv6, default: `localhost`) — Target master Redis host
* `port` (type: _integer_, allowed: TCP port, default: `6379`) — Target master Redis TCP port
* `password` (type: _string_, allowed: password values, default: none) — Master Redis password (if no password, do not set this key)

**[[redis.rescue]]**

* `host` (type: _string_, allowed: hostname, IPv4, IPv6, default: `localhost`) — Read-only rescue Redis host
* `port` (type: _integer_, allowed: TCP port, default: `6379`) — Read-only rescue Redis TCP port
* `password` (type: _string_, allowed: password values, default: none) — Read-only rescue Redis password (if no password, do not set this key)

### Initialize GeoIP

As Constellation does not distribute a GeoIP database in its repository, you will need to fetch it from MaxMind before you run Constellation for the first time (Constellation will refuse to start otherwise).

Execute the provided script:

`./scripts/init_geoip.sh --license_key=YOUR_GEOLITE2_LICENSE_KEY`

_YOUR_GEOLITE2_LICENSE_KEY should be replaced by a valid GeoLite2 license key. Please [follow instructions](https://dev.maxmind.com/geoip/geoip2/geolite2/) provided by MaxMind to obtain a license key._

_Note that once Constellation started from the GeoIP database you have manually initialized, it will keep the database up-to-date by checking and applying updates automatically in the background. The database initialization is a one-time operation. Make sure your license key is also set in the GeoIP update URL in the configuration._

### Run Constellation

Constellation can be run as such:

`./constellation -c /path/to/config.cfg`

### Test Constellation

Once running, DNS queries can be made against Constellation over the local network (using the default configuration):

`dig subdomain.relay.crisp.chat @::1`

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

* `zone_name`: The zone name (ie. base domain), eg. `relay.crisp.chat`
* `record_name`: The record name to read or alter (ie. sub-domain or base domain), eg. `client.@` for the `client.relay.crisp.chat` FQDN, or `@` for the `relay.crisp.chat` FQDN
* `record_type`: The DNS record type to read or alter for the `record_name`; either: `a`, `aaaa`, `cname`, `mx`, `txt`, `caa` or `ptr` ([open an issue](https://github.com/valeriansaliou/constellation/issues) if you need support for another record type)

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

**Rescue records for health-check:**

In case you are using health-check on the domain for zone, you may want to specify rescue records, that are served to DNS clients in the event all regular records (standard and Geo-DNS) are seen as dead. You can set the `rescue` property in the API to ensure failover servers are served, and connected to only in the event of a failure of default servers.

_If you do not set any `rescue` records; in the event all regular records get reported as dead, DNS clients will be served an empty response. Thus, it is judicious that you still serve fallback records._

**CNAME flattening:**

CNAMEs are handy to centralize record values in a single DNS entry, and re-use it across multiple DNS CNAME entries. It has its caveats, as for instance, it is illegal as per the DNS RFC to share it with other records on the same sub-domain. It is also illegal to setup a CNAME at the root of a domain. Furthermore, CNAMEs require DNS resolvers to perform a second resolving step as to resolve the flat value (eg. `A`, `AAAA`, `TXT`, etc. records), which is not super efficient as it adds extraneous latency when users resolve a domain using a CNAME.

CNAME flattening can help if you encounter an edge case of the DNS RFC with a CNAME record type. It lets Constellation resolve the actual flat value, and serve it right away, instead of returning the actual CNAME. CNAME flattening can be enabled for a record by setting the `flatten` property in the API to `true`. By default, no CNAME flattening is performed.

A dedicated Constellation thread manages previously-flattened CNAME values, and updates them as they change on their remote DNS server. As well, if a cached flattened CNAME has not been used for a long time, it is expunged from cache. Note that, for the sake of answering ASAP to the user, if a CNAME value with flattening enabled is not yet in cache, then Constellation will answer with the CNAME back, and delegate a deferred flatten order to the flattening manager thread, in order to avoid slowing down the initial response to the requesting user. Once the flattening manager thread has done its work, further DNS queries to the CNAME will then be answered with the flattened value (eg. it will return flat `A` record values, instead of the `CNAME` value).

_Note that the `flatten` option is only applicable to records with CNAME values. If flattening is enabled on eg. a `A` record type, the `flatten` property will have no effect._

#### API routes

##### Check if a DNS record exists

`HTTP HEAD http://constellation.local:8080/zone/<zone_name>/record/<record_name>/<record_type>/`

**Example request:**

```http
HEAD /zone/relay.crisp.chat/record/@/a HTTP/1.1
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
GET /zone/relay.crisp.chat/record/@/a HTTP/1.1
Authorization: Basic OlJFUExBQ0VfVEhJU19XSVRIX0FfU0VDUkVUX0tFWQ==
```

**Example response:**

```http
HTTP/1.1 200 OK
Content-Type: application/json

{"type":"a","name":"@","ttl":600,"blackhole": null,"regions": null,"values":["159.89.97.13","46.101.18.133"]}
```

##### Write a DNS record (or overwrite existing)

`HTTP PUT http://constellation.local:8080/zone/<zone_name>/record/<record_name>/<record_type>/`

**Example request (standard):**

```http
PUT /zone/relay.crisp.chat/record/@/a HTTP/1.1
Authorization: Basic OlJFUExBQ0VfVEhJU19XSVRIX0FfU0VDUkVUX0tFWQ==
Content-Type: application/json; charset=utf-8

{"values":["159.89.97.13","46.101.18.133"],"ttl":600}
```

**Example request (Geo-DNS):**

```http
PUT /zone/relay.crisp.chat/record/@/cname HTTP/1.1
Authorization: Basic OlJFUExBQ0VfVEhJU19XSVRIX0FfU0VDUkVUX0tFWQ==
Content-Type: application/json; charset=utf-8

{"regions":{"nnam":["client.nnam.geo.relay.crisp.net"],"snam":["client.snam.geo.relay.crisp.net"],"nsam":["client.nsam.geo.relay.crisp.net"],"ssam":["client.ssam.geo.relay.crisp.net"],"weu":["client.weu.geo.relay.crisp.net"],"ceu":["client.ceu.geo.relay.crisp.net"],"eeu":["client.eeu.geo.relay.crisp.net"],"ru":["client.ru.geo.relay.crisp.net"],"me":["client.me.geo.relay.crisp.net"],"naf":["client.naf.geo.relay.crisp.net"],"maf":["client.maf.geo.relay.crisp.net"],"saf":["client.saf.geo.relay.crisp.net"],"in":["client.in.geo.relay.crisp.net"],"seas":["client.seas.geo.relay.crisp.net"],"neas":["client.neas.geo.relay.crisp.net"],"oc":["client.oc.geo.relay.crisp.net"]},"values":["client.default.geo.relay.crisp.net"],"ttl":600}
```

**Example request (health-checked):**

```http
PUT /zone/relay.crisp.chat/record/@/a HTTP/1.1
Authorization: Basic OlJFUExBQ0VfVEhJU19XSVRIX0FfU0VDUkVUX0tFWQ==
Content-Type: application/json; charset=utf-8

{"values":["159.89.97.13","46.101.18.133"],"rescue":["139.59.174.13"],"ttl":60}
```

**Example response:**

```http
HTTP/1.1 200 OK
```

**Example request (CNAME-flattened):**

```http
PUT /zone/relay.crisp.chat/record/@/cname HTTP/1.1
Authorization: Basic OlJFUExBQ0VfVEhJU19XSVRIX0FfU0VDUkVUX0tFWQ==
Content-Type: application/json; charset=utf-8

{"values":["alias.crisp.net"],"flatten":true,"ttl":60}
```

**Example response:**

```http
HTTP/1.1 200 OK
```

##### Delete a DNS record

`HTTP DELETE http://constellation.local:8080/zone/<zone_name>/record/<record_name>/<record_type>/`

**Example request:**

```http
DELETE /zone/relay.crisp.chat/record/@/a HTTP/1.1
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

* `zone_name`: The zone name (ie. base domain), eg. `relay.crisp.chat`
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
GET /zone/relay.crisp.chat/metrics/5m/query/origins HTTP/1.1
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
