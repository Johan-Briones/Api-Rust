use postgres::{ Client, NoTls };
use postgres::Error as PostgresError;
use std::net::{ TcpListener, TcpStream };
use std::io::{ Read, Write };
use std::env;
use serde::{Serialize, Deserialize};

//#[macro_use]
//extern crate serde_derive;

//Model: USer struct with id, nombre, clave
#[derive(Serialize, Deserialize, Debug)]
struct Carrera {
    id: Option<i32>,
    name: String,
    key: String,
}

//DATABASE_URL
//const DB_URL: &str = env!("DATABASE_URL");

//constants
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const INTERNAL_SERVER_ERROR: &str = "HTTP/1.1 500 INTERNAL SERVER ERROR\r\n\r\n";

//main function
fn main() {

    let db_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            println!("Error: DATABASE_URL variable de entorno no está definida.");
            return;
        }
    };


    //Set database
    if let Err(e) = set_database(&db_url) {
        println!("Error: {}", e);
        return;
    }

    //start server and print port
    let listener = TcpListener::bind(format!("0.0.0.0:8080")).unwrap();
    println!("Server started at port 8080");

    //handle the client
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_carreras(stream, &db_url);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}

//handle_carreras function
fn handle_carreras(mut stream: TcpStream, db_url: &str) {
    let mut buffer = [0; 1024];
    let mut request = String::new();

    match stream.read(&mut buffer) {
        Ok(size) => {
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

            let (status_line, content) = match &*request {
                r if r.starts_with("POST /carreras") => handle_post_request(r, db_url),
                r if r.starts_with("GET /carreras/") => handle_get_request(r, db_url),
                r if r.starts_with("GET /carreras") => handle_get_all_request(r, db_url),
                r if r.starts_with("PUT /carreras/") => handle_put_request(r, db_url),
                r if r.starts_with("DELETE /carreras/") => handle_delete_request(r, db_url),
                _ => (NOT_FOUND.to_string(), "404 Not Found".to_string()),
            };

            stream.write_all(format!("{}{}", status_line, content).as_bytes()).unwrap();
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

//CONTROLLERS

//handle_post_request function
fn handle_post_request(request: &str, db_url: &str) -> (String, String) {
    match (get_carrera_request_body(&request), Client::connect(db_url, NoTls)) {
        (Ok(carrera), Ok(mut client)) => {
            client
                .execute(
                    "INSERT INTO carreras (name, key) VALUES ($1, $2)",
                    &[&carrera.name, &carrera.key]
                )
                .unwrap();

            (OK_RESPONSE.to_string(), "Carrea añadida con éxito".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

//handle_get_request function
fn handle_get_request(request: &str, db_url: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(), Client::connect(db_url, NoTls)) {
        (Ok(id), Ok(mut client)) =>
            match client.query_one("SELECT * FROM carreras WHERE id = $1", &[&id]) {
                Ok(row) => {
                    let carrera = Carrera {
                        id: row.get(0),
                        name: row.get(1),
                        key: row.get(2),
                    };

                    (OK_RESPONSE.to_string(), serde_json::to_string(&carrera).unwrap())
                }
                _ => (NOT_FOUND.to_string(), "Carrera no encontrada".to_string()),
            }

        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

//handle_get_all_request function
fn handle_get_all_request(_request: &str, db_url: &str) -> (String, String) {
    match Client::connect(db_url, NoTls) {
        Ok(mut client) => {
            let mut carreras = Vec::new();

            for row in client.query("SELECT * FROM carreras", &[]).unwrap() {
                carreras.push(Carrera {
                    id: row.get(0),
                    name: row.get(1),
                    key: row.get(2),
                });
            }

            (OK_RESPONSE.to_string(), serde_json::to_string(&carreras).unwrap())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

//handle_put_request function
fn handle_put_request(request: &str, db_url: &str) -> (String, String) {
    match
        (
            get_id(&request).parse::<i32>(),
            get_carrera_request_body(&request),
            Client::connect(db_url, NoTls),
        )
    {
        (Ok(id), Ok(carrera), Ok(mut client)) => {
            client
                .execute(
                    "UPDATE carreras SET name = $1, key = $2 WHERE id = $3",
                    &[&carrera.name, &carrera.key, &id]
                )
                .unwrap();

            (OK_RESPONSE.to_string(), "Carrera actualizada".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

//handle_delete_request function
fn handle_delete_request(request: &str, db_url: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(), Client::connect(db_url, NoTls)) {
        (Ok(id), Ok(mut client)) => {
            let rows_affected = client.execute("DELETE FROM carreras WHERE id = $1", &[&id]).unwrap();

            if rows_affected == 0 {
                return (NOT_FOUND.to_string(), "Carrera no encontrada".to_string());
            }

            (OK_RESPONSE.to_string(), "Carrera eliminada".to_string())
        }
        _ => (INTERNAL_SERVER_ERROR.to_string(), "Error".to_string()),
    }
}

//set_database function
fn set_database(db_url: &str) -> Result<(), PostgresError> {
    //Connect to database
    let mut client = Client::connect(db_url, NoTls)?;

    //Create table
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS carreras (
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            key VARCHAR NOT NULL
        )"
    )?;
    Ok(())
}

//get_id function
fn get_id(request: &str) -> &str {
    request.split("/").nth(2).unwrap_or_default().split_whitespace().next().unwrap_or_default()
}

//deserialize carrera from request body with the id
fn get_carrera_request_body(request: &str) -> Result<Carrera, serde_json::Error> {
    serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
}











