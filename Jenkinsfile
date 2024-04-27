pipeline {
    agent any
    
    environment {
        RUSTAPP_IMAGE_NAME = "francescoxx/rustapp:1.0.0"
        DB_IMAGE_NAME = "postgres:12"
        RUSTAPP_CONTAINER_NAME = "rustapp"
        DB_CONTAINER_NAME = "db"
        DB_NAME = "postgres" // Define el nombre de tu base de datos
        DB_USER = "postgres" // Define el usuario de tu base de datos
        DB_PASSWORD = "postgres" // Define la contraseña de tu base de datos
        DATABASE_URL = "${env.DATABASE_URL}" // Asigna el valor de la variable de entorno DATABASE_URL
    }

    stages {
        stage("Preparation") {
            steps {
                git "https://github.com/Johan-Briones/Api-Rust.git"
            }
        }
        
        stage("Build Rust App") {
            steps {
                script {
                    sh "docker build -t $RUSTAPP_IMAGE_NAME ."
                }
            }
        }
        
        stage("Test Rust App") {
            steps {
                script {
                    sh "cargo test"
                }
            }
        }

        stage("Build Database") {
            steps {
                script {
                    sh "docker pull $DB_IMAGE_NAME"
                }
            }
        }

        stage("Deploy") {
            steps {
                script {
                    // Ejecutar contenedor de PostgreSQL
                    sh "docker run -d --name $DB_CONTAINER_NAME -e POSTGRES_DB=$DB_NAME -e POSTGRES_USER=$DB_USER -e POSTGRES_PASSWORD=$DB_PASSWORD $DB_IMAGE_NAME"

                    // Esperar unos segundos para asegurarse de que el contenedor de la base de datos esté en funcionamiento
                    sleep 10

                    // Ejecutar contenedor de la API Rust
                    sh "docker run -d --name $RUSTAPP_CONTAINER_NAME -p 8080:8080 --link $DB_CONTAINER_NAME:postgres $RUSTAPP_IMAGE_NAME"
                }
            }
        }
    }
}








