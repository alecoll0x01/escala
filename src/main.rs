use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

const DIAS_UTEIS: usize = 5;

#[derive(Serialize, Deserialize, Clone)]
struct Escala {
    semanas: Vec<Semana>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Semana {
    dias: Vec<String>,
    remotos: Vec<String>,
}

#[derive(Deserialize)]
struct GerarEscalaRequest {
    funcionarios: Vec<String>,
    num_semanas: usize,
}

impl Escala {
    fn new(funcionarios: &[String], num_semanas: usize) -> Self {
        let mut contagem_presencial = HashMap::new();
        funcionarios.iter().for_each(|f| {
            contagem_presencial.insert(f, 0);
        });

        let semanas = (0..num_semanas)
            .map(|_| Semana::gerar(funcionarios, &mut contagem_presencial))
            .collect();

        Escala { semanas }
    }
}

impl Semana {
    fn gerar(funcionarios: &[String], contagem_presencial: &mut HashMap<&String, usize>) -> Self {
        let mut rng = rand::thread_rng();
        let mut funcionarios_disponiveis = funcionarios.to_vec();
        funcionarios_disponiveis.shuffle(&mut rng);

        let mut dias = Vec::with_capacity(DIAS_UTEIS);
        for _ in 0..DIAS_UTEIS.min(funcionarios.len()) {
            if let Some(funcionario) = funcionarios_disponiveis.pop() {
                dias.push(funcionario.clone());
                *contagem_presencial.get_mut(&funcionario).unwrap() += 1;
            }
        }

        let remotos = funcionarios
            .iter()
            .filter(|f| !dias.contains(f))
            .cloned()
            .collect();

        Semana { dias, remotos }
    }
}

struct AppState {
    escala: Mutex<Option<Escala>>,
}

async fn gerar_escala(
    data: web::Data<AppState>,
    req: web::Json<GerarEscalaRequest>,
) -> impl Responder {
    let mut escala = data.escala.lock().unwrap();
    if req.funcionarios.is_empty() {
        return HttpResponse::BadRequest().json("A lista de funcionários não pode estar vazia");
    }
    *escala = Some(Escala::new(&req.funcionarios, req.num_semanas));
    HttpResponse::Ok().json("Escala gerada com sucesso")
}

async fn obter_escala(data: web::Data<AppState>) -> impl Responder {
    let escala = data.escala.lock().unwrap();
    match &*escala {
        Some(e) => HttpResponse::Ok().json(e),
        None => HttpResponse::NotFound().json("Escala não encontrada. Gere uma primeira."),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = web::Data::new(AppState {
        escala: Mutex::new(None),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/gerar-escala", web::post().to(gerar_escala))
            .route("/obter-escala", web::get().to(obter_escala))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}