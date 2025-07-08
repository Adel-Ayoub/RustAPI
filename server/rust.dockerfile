FROM rust:latest as builder
WORKDIR /app
ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL
COPY . .
RUN cargo build --release

FROM ubuntu:22.04
RUN apt-get update && apt-get install -y ca-certificates postgresql-client && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/local/bin
COPY --from=builder /app/target/release/server .

# Add a simple wait script
RUN echo '#!/bin/bash\nuntil pg_isready -h database -p 5432 -U adel; do\n  echo "Waiting for database..."\n  sleep 2\ndone\necho "Database is ready!"\nexec ./server' > wait-for-db.sh
RUN chmod +x wait-for-db.sh

CMD ["./wait-for-db.sh"]
