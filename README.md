# et
Expense Tracker

## Deploying

### Requirements

Docker
Postgres

### Migrations

Migrations are handled with Atlas.

Due to Postgres Extentions being for logged-in users a manual command is required on initial deploy

`CREATE EXTENSION IF NOT EXISTS ltree;`
