# et
Expense Tracker

An expense tracker that syncs account data using the [SimpleFin](https://www.simplefin.org/).

Track expenses with an arbitrary number of tags instead of a single category. Compare spending based on tags.

## Deploying

### Requirements

Docker
Postgres
OIDC


### Configuration

et.toml

```
database_url="postgres://USER:PASSWORD@HOST:PORT/DB?sslmode=verify-full"

[auth]
issuer_url = "…"
redirect_url = "https://HOST/oidc/login_auth"
client_secret = "CLIENT_SECRET"
client_id = "CLIENT_ID"
key = "COOKIE_SESSION_KEY"
```


#### Features

A `features` block can be used to enable/disable features

`charts` - alpha feature

```
[features]
charts = true
```

### Migrations

Migrations are handled with `et-migrate` tool included in this package/Docker image

Using the Docker image

To see what migrations will run

`/usr/local/bin/et-migrate --config-file [FILE] print`

To run migrations

`/usr/local/bin/et-migrate --config-file [FILE] migrate`
