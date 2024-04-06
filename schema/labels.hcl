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

