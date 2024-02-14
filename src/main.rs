use axum::{
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::NaiveDateTime;
use persistence::PostgresRepository;
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};

#[derive(Deserialize, Serialize)]
#[serde(try_from = "String")]
pub struct TransactionType(String);

#[derive(Deserialize, Serialize)]
#[serde(try_from = "String")]
pub struct TransactionDescription(String);

#[derive(Deserialize, Serialize)]
#[serde(try_from = "i64")]
pub struct TransactionValue(i64);

impl TryFrom<String> for TransactionType {
    type Error = &'static str;

    fn try_from(tipo: String) -> Result<Self, Self::Error> {
        if tipo == "d" || tipo == "c" {
            Ok(TransactionType(tipo))
        } else {
            Err("transaction type invalid")
        }
    }
}

impl TryFrom<String> for TransactionDescription {
    type Error = &'static str;

    fn try_from(descricao: String) -> Result<Self, Self::Error> {
        if descricao.len() > 0 && descricao.len() <= 10 {
            Ok(TransactionDescription(descricao))
        } else {
            Err("transaction description invalid")
        }
    }
}

impl TryFrom<i64> for TransactionValue {
    type Error = &'static str;

    fn try_from(valor: i64) -> Result<Self, Self::Error> {
        Ok(TransactionValue(valor * 100))
    }
}

#[derive(Deserialize, Serialize)]
struct Transaction {
    valor: TransactionValue,
    tipo: TransactionType,
    descricao: TransactionDescription,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
struct Balance {
    saldo_atual: i64,
    limite: i64,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
struct Extract {
    valor: i64,
    tipo: String,
    descricao: String,
    realizada_em: NaiveDateTime,
}

#[derive(Deserialize, Serialize, sqlx::FromRow)]
struct Client {
    id: i32,
}

mod persistence;

#[tokio::main]
async fn main() {
    let repository = PostgresRepository::connect(env::var("DATABASE_URL").unwrap()).await;

    let repository_state = Arc::new(repository);

    let app = Router::new()
        .route("/clientes/:id/extrato", get(get_extract))
        .route("/clientes/:id/transacoes", post(create_transaction))
        .layer(Extension(repository_state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize)]
struct BalanceResponse {
    total: i64,
    limite: i64,
    data_extrato: NaiveDateTime,
}
#[derive(Serialize)]
struct ExtractResponse {
    saldo: BalanceResponse,
    ultimas_transacoes: Vec<Extract>,
}
async fn get_extract(
    Extension(repository): Extension<Arc<PostgresRepository>>,
    Path(id): Path<String>,
) -> Result<Json<ExtractResponse>, StatusCode> {
    let client_exists = repository.client_exists(id.parse().unwrap()).await.unwrap();

    if !client_exists {
        return Err(StatusCode::NOT_FOUND);
    }

    let current_balance = repository
        .get_balance(id.parse().unwrap())
        .await
        .unwrap()
        .unwrap();
    let extract = repository.get_extract(id.parse().unwrap()).await.unwrap();

    let response = ExtractResponse {
        saldo: BalanceResponse {
            total: current_balance.saldo_atual,
            limite: current_balance.limite,
            data_extrato: chrono::Local::now().naive_local(),
        },
        ultimas_transacoes: extract,
    };
    Ok(Json(response))
}

async fn create_transaction(
    Extension(repository): Extension<Arc<PostgresRepository>>,
    Path(id): Path<String>,
    Json(payload): Json<Transaction>,
) -> impl IntoResponse {
    let client_exists = repository.client_exists(id.parse().unwrap()).await.unwrap();

    if client_exists {
        let user_information = repository
            .get_balance(id.parse().unwrap())
            .await
            .unwrap()
            .unwrap();
        let new_balance: i64;

        if payload.tipo.0 == "d" {
            new_balance = user_information.saldo_atual - payload.valor.0;
        } else {
            new_balance = user_information.saldo_atual + payload.valor.0;
        }

        if new_balance < -user_information.limite {
            return StatusCode::UNPROCESSABLE_ENTITY;
        }

        repository
            .create_transaction(id.parse().unwrap(), payload)
            .await
            .unwrap();
        repository
            .update_client_balance(id.parse().unwrap(), new_balance)
            .await
            .unwrap();
        return StatusCode::OK;
    } else {
        return StatusCode::NOT_FOUND;
    }
}
