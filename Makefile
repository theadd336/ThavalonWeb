.PHONY: all

all:
	docker-compose -f docker-compose.yaml -f docker-compose.webapp.yaml up

.PHONY: api
api:
	docker-compose -f docker-compose.yaml up

.PHONY: web
web:
	docker-compose -f docker-compose.webapp.yaml up

