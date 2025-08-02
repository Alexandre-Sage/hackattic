#!/bin/env sh

BASE_URL="https://hackattic.com/challenges/backup_restore"
PROBLEM_URL="$BASE_URL/problem?access_token=$AUTH_TOKEN"
SOLVE_URL="$BASE_URL/solve?access_token=$AUTH_TOKEN"

DATABASE="backup_restore"
QUERY='SELECT TRIM("ssn")
	FROM criminal_records
	WHERE status = '\''alive'\'';' 

docker run --rm -d -p 5432:5432 \
	--name="$DATABASE" \
	-e POSTGRES_DB="$DATABASE" \
	-e POSTGRES_PASSWORD="root" \
		postgres

until docker exec "$DATABASE" pg_isready  -U "$PG_USER" -d "$DATABASE" > /dev/null 2>&1; do
    echo "Waiting for PostgreSQL to be ready..."
    sleep 1
done

echo "PostgreSQL is up and accepting connections!"

curl "$PROBLEM_URL" | jq -r .dump | \
	 base64 --decode | \
	 gunzip -c | \
         docker exec -i $DATABASE \
	 psql -U postgres \
	 -d $DATABASE && \
         docker exec  $DATABASE psql -U postgres \
	 -d $DATABASE -t -c "$QUERY" | \
	 jq -sR 'split("\n") | 
	 	map(select(length > 0) | trim) | 
		{ "alive_ssns": . }' | \
	 curl -X POST "$SOLVE_URL" \
	 -H "Content-Type: application/json" \
	 -d @-

docker container kill $DATABASE
