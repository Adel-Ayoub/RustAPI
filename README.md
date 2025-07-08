# RustAPI
## A high-performance REST API built with Rust and PostgreSQL
> Task management API with Docker containerization.

## How to Install RustAPI
### From Source
```bash
git clone https://github.com/Adel-Ayoub/RustAPI.git
cd RustAPI
docker-compose up --build
```

## API Endpoints
| Method | Endpoint | Description |
| ------ | -------- | ----------- |
| **GET** | `/api/tasks` | Get all tasks |
| **GET** | `/api/tasks/{id}` | Get specific task |
| **POST** | `/api/tasks` | Create new task |
| **PUT** | `/api/tasks/{id}` | Update existing task |
| **DELETE** | `/api/tasks/{id}` | Delete task |

## Technical Stack
- **Rust** - Systems programming language for performance and safety
- **PostgreSQL** - Relational database with ACID compliance
- **Docker** - Containerization and orchestration
- **Serde** - Serialization and deserialization framework

## Requirements
- Docker and Docker Compose
- Rust 1.70+ (for local development)
- PostgreSQL (for local development)

## Configuration
| Setting | Value |
| ------- | ----- |
| **API Port** | 3000 |
| **Database Port** | 5433 (external) |
| **Database User** | adel |
| **Database Name** | rustapidb |

## Local Development
### Option 1: Using Docker database
```bash
# Start only the database
docker-compose up database
```

```bash
# Set environment variable to connect to Docker database
export DATABASE_URL="postgres://adel:adel123@localhost:5433/rustapidb"
```

```bash
# Run API locally
cd server
cargo run
```

### Option 2: Using your own PostgreSQL
```bash
# Set environment variable to your local database
export DATABASE_URL="postgres://your_user:your_pass@localhost:5432/your_db"
```

```bash
# Run API locally
cd server
cargo run
```

## Example Usage
```bash
# Create a task
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "Learn Rust", "description": "Build APIs with Rust", "completed": false}'
```

```bash
# Get all tasks
curl http://localhost:3000/api/tasks
```

```bash
# Update a task
curl -X PUT http://localhost:3000/api/tasks/1 \
  -H "Content-Type: application/json" \
  -d '{"title": "Learn Rust - Done", "description": "Completed Rust API tutorial", "completed": true}'
```
