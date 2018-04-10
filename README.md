Constellation
=============

[![Build Status](https://travis-ci.org/valeriansaliou/constellation.svg?branch=master)](https://travis-ci.org/valeriansaliou/constellation)

**Pluggable authoritative DNS server. Entries can be added & removed from an HTTP REST API.**

Constellation is a small authoritative server that lets you manage DNS entries from an HTTP REST API, in a generic way. It can be plugged to your existing infrastructure to manage DNS records for users of your service, eg. to configure outbound email records that cannot be easily wildcarded in a traditional DNS server (DKIM, DMARC, SPF records).

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

## :fire: Report A Vulnerability

If you find a vulnerability in Constellation, you are more than welcome to report it directly to [@valeriansaliou](https://github.com/valeriansaliou) by sending an encrypted email to [valerian@valeriansaliou.name](mailto:valerian@valeriansaliou.name). Do not report vulnerabilities in public GitHub issues, as they may be exploited by malicious people to target production servers running an unpatched Constellation instance.

**:warning: You must encrypt your email using [@valeriansaliou](https://github.com/valeriansaliou) GPG public key: [:key:valeriansaliou.gpg.pub.asc](https://valeriansaliou.name/files/keys/valeriansaliou.gpg.pub.asc).**

**:gift: Based on the severity of the vulnerability, I may offer a $200 (US) bounty to whomever reported it.**
