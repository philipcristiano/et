CREATE EXTENSION ltree;

CREATE TABLE et_user (
    id character varying NOT NULL,
    name character varying NOT NULL,
    CONSTRAINT folio_user_pkey PRIMARY KEY (id)
);

CREATE TABLE simplefin_connections (
    id uuid NOT NULL,
    access_url varchar NOT NULL,
    CONSTRAINT simplefin_connections_pkey PRIMARY KEY (id)
);

CREATE TABLE simplefin_connection_sync_info (
    connection_id uuid NOT NULL,
    ts timestamptz NOT NULL,
    CONSTRAINT simplefin_connections_sync_info_pkey PRIMARY KEY (connection_id, ts),
    CONSTRAINT simplefin_connection FOREIGN KEY (connection_id) REFERENCES simplefin_connections (id) ON DELETE CASCADE
);

CREATE TABLE simplefin_connection_sync_errors (

    connection_id uuid NOT NULL,
    ts timestamptz NOT NULL,
    message varchar NOT NULL,

    CONSTRAINT simplefin_connection_sync_errors_pkey PRIMARY KEY (connection_id, ts),
    CONSTRAINT simplefin_connection_sync_error_fk_connection_id FOREIGN KEY (connection_id) REFERENCES simplefin_connections (id)
);

CREATE TABLE simplefin_accounts (

    id uuid NOT NULL DEFAULT gen_random_uuid(),
    connection_id uuid NOT NULL,
    simplefin_id varchar NOT NULL,
    name varchar NOT NULL,
    currency varchar NOT NULL,
    active bool NOT NULL DEFAULT true,
    custom_name varchar,

    CONSTRAINT simplefin_accounts_pkey PRIMARY KEY (id),
    CONSTRAINT simplefin_accounts_fk_connection_id FOREIGN KEY (connection_id) REFERENCES simplefin_connections (id)
);

CREATE UNIQUE INDEX idx_connection_source_id ON public.simplefin_accounts USING btree (connection_id, simplefin_id);

CREATE TABLE simplefin_account_balances (
    account_id uuid NOT NULL,
    ts timestamptz NOT NULL,
    balance money NOT NULL,

    CONSTRAINT simplefin_account_balances_pkey PRIMARY KEY (account_id, ts),
    CONSTRAINT simplefin_account_balances_fk_account_id FOREIGN KEY (account_id) REFERENCES simplefin_accounts (id)

);

CREATE INDEX idx_account_ts ON public.simplefin_account_balances USING btree (account_id, ts DESC);

CREATE TABLE simplefin_account_transactions (
    id uuid NOT NULL DEFAULT gen_random_uuid(),
    account_id uuid NOT NULL,
    simplefin_id varchar NOT NULL,
    posted timestamptz NOT NULL,
    amount money NOT NULL,
    transacted_at timestamptz,
    pending bool,
    description varchar NOT NULL,

    CONSTRAINT simplefin_account_transactions_pkey PRIMARY KEY (id),
    CONSTRAINT simplefin_account_transaction_fk_account_id FOREIGN KEY (account_id) REFERENCES simplefin_accounts (id)
);

CREATE UNIQUE INDEX idx_account_source_id ON public.simplefin_account_transactions USING btree (account_id, simplefin_id);

CREATE TABLE labels (
    id uuid NOT NULL,
    label ltree NOT NULL,
    CONSTRAINT labels_pkey PRIMARY KEY (id)
);

CREATE UNIQUE INDEX idx_label_path ON public.labels USING btree (label);

CREATE TABLE transaction_labels (
    transaction_id uuid NOT NULL,
    label_id uuid NOT NULL,
    CONSTRAINT transaction_labels_pkey PRIMARY KEY (transaction_id, label_id),
    CONSTRAINT transaction_labels_fk_transaction_id FOREIGN KEY (transaction_id) REFERENCES simplefin_account_transactions (id) ON DELETE CASCADE,
    CONSTRAINT transaction_labels_fk_label_id FOREIGN KEY (label_id) REFERENCES labels (id) ON DELETE CASCADE
);

CREATE TABLE rules (
    id uuid NOT NULL,
    name text NOT NULL,
    transaction_filter_qs text NOT NULL,

    CONSTRAINT rules_pkey PRIMARY KEY (id)
);

CREATE TABLE rules_labels (
    rule_id uuid NOT NULL,
    label_id uuid NOT NULL,

    CONSTRAINT rules_labels_pkey PRIMARY KEY (rule_id, label_id),
    CONSTRAINT rules_labels_rule_id FOREIGN KEY (rule_id) REFERENCES rules (id) ON DELETE CASCADE,
    CONSTRAINT rules_labels_label_id FOREIGN KEY (label_id) REFERENCES labels (id) ON DELETE CASCADE
);

