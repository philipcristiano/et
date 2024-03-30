schema "public" {
  comment = "A schema comment"
}

table "simplefin_connections" {
  schema = schema.public
  column "id" {
    type = uuid
  }
  column "access_url" {
    type = varchar
  }
  column "user_id" {
    type = varchar
  }
  primary_key {
    columns = [
      column.id
    ]
  }
  foreign_key "simplefin_connection_user" {
    columns = [column.user_id]
    ref_columns = [table.et_user.column.id]
    on_delete = CASCADE
    on_update = NO_ACTION
  }
}

table "simplefin_accounts" {
  schema = schema.public

  column "connection_id" {
    type = uuid
  }
  column "id" {
    type = varchar
  }
  column "name" {
    type = varchar
  }
  column "currency" {
    type = varchar
  }
  column "user_id" {
    type = varchar
  }
  primary_key {
    columns = [
      column.id,
      column.connection_id,
    ]
  }

  foreign_key "simplefin_connection" {
    columns = [column.connection_id]
    ref_columns = [table.simplefin_connections.column.id]
    on_delete = CASCADE
    on_update = NO_ACTION
  }
  foreign_key "simplefin_account_user" {
    columns = [column.user_id]
    ref_columns = [table.et_user.column.id]
    on_delete = CASCADE
    on_update = NO_ACTION
  }
}

table "simplefin_account_balances" {
  schema = schema.public

  column "connection_id" {
    type = uuid
  }
  column "account_id" {
    type = varchar
  }
  column "ts" {
    type = timestamp
  }
  column "balance" {
    type = money
  }
  primary_key {
    columns = [
      column.account_id,
      column.connection_id,
      column.ts,
    ]
  }

  foreign_key "simplefin_account" {
    columns = [
        column.connection_id,
        column.account_id
    ]
    ref_columns = [
        table.simplefin_accounts.column.connection_id,
        table.simplefin_accounts.column.id
    ]
    on_delete = CASCADE
    on_update = NO_ACTION
  }
}
