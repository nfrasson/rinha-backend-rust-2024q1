use crate::{Balance, Client, Extract, Transaction};
use sqlx::postgres::PgPool;
use sqlx::postgres::PgPoolOptions;

pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    pub async fn connect(url: String) -> Self {
        PostgresRepository {
            pool: PgPoolOptions::new()
                .min_connections(30)
                .connect(&url)
                .await
                .expect("error connecting to database"),
        }
    }

    pub async fn get_balance(&self, client_id: i32) -> Result<Option<Balance>, sqlx::Error> {
        sqlx::query_as("SELECT saldo_atual, limite FROM Clientes WHERE id = $1")
            .bind(client_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn get_extract(&self, client_id: i32) -> Result<Vec<Extract>, sqlx::Error> {
        sqlx::query_as(
            "SELECT valor, tipo, descricao, realizada_em FROM Transacoes WHERE cliente_id = $1 ORDER BY realizada_em DESC LIMIT 10;",
        ).bind(client_id)
        .fetch_all(&self.pool)
        .await
    }

    pub async fn client_exists(&self, client_id: i32) -> Option<bool> {
        let client: Option<Client> = sqlx::query_as("SELECT id FROM Clientes WHERE id = $1")
            .bind(client_id)
            .fetch_optional(&self.pool)
            .await
            .unwrap();

        match client {
            Some(_) => Some(client_id == client.unwrap().id),
            None => Some(false),
        }
    }

    pub async fn update_client_balance(
        &self,
        client_id: i32,
        new_balance: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE Clientes SET saldo_atual = $1 WHERE id = $2")
            .bind(new_balance)
            .bind(client_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn create_transaction(
        &self,
        client_id: i32,
        transaction: Transaction,
    ) -> Result<(), sqlx::Error> {
        let _ = sqlx::query(
            "INSERT INTO Transacoes (cliente_id, valor, tipo, descricao) VALUES ($1, $2, $3, $4)",
        )
        .bind(client_id)
        .bind(transaction.valor.0)
        .bind(transaction.tipo.0)
        .bind(transaction.descricao.0)
        .execute(&self.pool)
        .await;

        Ok(())
    }
}
