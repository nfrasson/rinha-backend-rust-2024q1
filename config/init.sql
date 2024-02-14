CREATE TABLE IF NOT EXISTS clientes (
    id SERIAL PRIMARY KEY,
    limite BIGINT NOT NULL,
    saldo_atual BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS transacoes (
    id SERIAL PRIMARY KEY,
    cliente_id INTEGER NOT NULL REFERENCES clientes(id),
    valor BIGINT NOT NULL,
    tipo CHAR(1) CHECK (tipo IN ('c', 'd')),
    descricao VARCHAR(10),
    realizada_em TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO clientes (id, limite, saldo_atual) VALUES
    (1, 100000, 0),
    (2, 80000, 0),
    (3, 1000000, 0),
    (4, 10000000, 0),
    (5, 500000, 0)
ON CONFLICT (id) DO NOTHING;