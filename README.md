# cf-ddns

## a simple CloudFlare DDNS client

## Usage

```
cf-ddns 0.1.0
An Cloudflare DDNS client.

USAGE:
    cf-ddns [FLAGS] --config <config>

FLAGS:
        --debug      enable debug log
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>    config path, - means read config from stdin
```

## Example config

```yaml
email: example@gmail.com
auth_key: 1234567890
domain: example.com
dns_type: AAAA
name: www.example.com
ttl: 30
proxied: false
```
