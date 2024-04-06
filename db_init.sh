createdb et
createuser --superuser et

psql -U et et -c "CREATE EXTENSION IF NOT EXISTS ltree;"
