table "labels" {
  schema = schema.public
  column "id" {
    type = uuid
  }
  column "label" {
    type = sql("ltree")
  }
  primary_key {
    columns = [
      column.id
    ]
  }
  index "idx_label_path" {
      on {
          column = column.label
      }
      unique = true
  }
}

table "transaction_labels" {
  schema = schema.public

  column "connection_id" {
    type = uuid
  }
  column "account_id" {
    type = varchar
  }
  column "transaction_id" {
    type = varchar
  }
  column "label_id" {
    type = uuid
  }

  primary_key {
    columns = [
      column.transaction_id,
      column.connection_id,
      column.account_id,
      column.label_id,
    ]
  }

  foreign_key "fk_simplefin_transaction" {
    columns = [
        column.connection_id,
        column.account_id,
        column.transaction_id,
    ]
    ref_columns = [
        table.simplefin_account_transactions.column.connection_id,
        table.simplefin_account_transactions.column.account_id,
        table.simplefin_account_transactions.column.id,
    ]
    on_delete = CASCADE
    on_update = NO_ACTION
  }
  foreign_key "fk_label" {
    columns = [
        column.label_id,
    ]
    ref_columns = [
        table.labels.column.id,
    ]
    on_delete = NO_ACTION
    on_update = NO_ACTION
  }
}
