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
  primary_key {
    columns = [
      column.id
    ]
  }
}

table "simplefin_connection_sync_info" {
  schema = schema.public

  column "connection_id" {
    type = uuid
  }
  column "ts" {
    type = timestamptz
  }

  primary_key {
    columns = [
      column.connection_id,
      column.ts,
    ]
  }

  foreign_key "simplefin_connection" {
    columns = [column.connection_id]
    ref_columns = [table.simplefin_connections.column.id]
    on_delete = CASCADE
    on_update = NO_ACTION
  }
}

table "simplefin_connection_sync_errors" {
  schema = schema.public

  column "connection_id" {
    type = uuid
  }
  column "ts" {
    type = timestamptz
  }
  column "message" {
    type = varchar
  }

  primary_key {
    columns = [
      column.connection_id,
      column.ts,
    ]
  }

  foreign_key "simplefin_connection" {
    columns = [column.connection_id, column.ts]
    ref_columns = [table.simplefin_connection_sync_info.column.connection_id, table.simplefin_connection_sync_info.column.ts]
    on_delete = CASCADE
    on_update = NO_ACTION
  }
}

table "simplefin_accounts" {
  schema = schema.public

  column "id" {
    type = uuid
    default = sql("gen_random_uuid()")
  }
  column "connection_id" {
    type = uuid
  }
  column "simplefin_id" {
    type = varchar
  }
  column "name" {
    type = varchar
  }
  column "currency" {
    type = varchar
  }
  primary_key {
    columns = [
      column.id,
    ]
  }

  index "idx_connection_source_id" {
      on {
          column = column.connection_id
      }
      on {
          column = column.simplefin_id
      }
      unique = true
  }

  foreign_key "simplefin_connection" {
    columns = [column.connection_id]
    ref_columns = [table.simplefin_connections.column.id]
    on_delete = CASCADE
    on_update = NO_ACTION
  }
}

table "simplefin_account_balances" {
  schema = schema.public

  column "account_id" {
    type = uuid
  }
  column "ts" {
    type = timestamptz
  }
  column "balance" {
    type = money
  }
  primary_key {
    columns = [
      column.account_id,
      column.ts,
    ]
  }

  index "idx_account_ts" {
      on {
          column = column.account_id
      }
      on {
          column = column.ts
          desc = true
      }
  }

  foreign_key "simplefin_account" {
    columns = [
        column.account_id
    ]
    ref_columns = [
        table.simplefin_accounts.column.id
    ]
    on_delete = CASCADE
    on_update = NO_ACTION
  }
}

table "simplefin_account_transactions" {
  schema = schema.public

  column "id" {
    type = uuid
    default = sql("gen_random_uuid()")
  }
  column "account_id" {
    type = uuid
  }
  column "simplefin_id" {
    type = varchar
  }
  column "posted" {
    type = timestamptz
  }
  column "amount" {
    type = money
  }
  column "transacted_at" {
    type = timestamptz
    null = true
  }
  column "pending" {
    type = bool
    null = true
  }
  column "description" {
    type = varchar
  }
  primary_key {
    columns = [
      column.id,
    ]
  }

  index "idx_account_source_id" {
      on {
          column = column.account_id
      }
      on {
          column = column.simplefin_id
      }
      unique = true
  }

  foreign_key "simplefin_account" {
    columns = [
        column.account_id
    ]
    ref_columns = [
        table.simplefin_accounts.column.id
    ]
    on_delete = CASCADE
    on_update = NO_ACTION
  }
}
