.PHONY: up down clean logs shell debug

up:
	docker-compose up -d --build

down:
	docker-compose down

clean:
	docker-compose down -v --remove-orphans

logs:
	docker-compose logs -f

shell:  # Get an interactive shell inside the container
	docker-compose run --rm app /bin/bash

debug:  # Run strace on the binary (use inside shell: strace -f /usr/local/bin/poker-server)
	@echo "Run this inside the shell (after 'make shell'):"
	@echo "strace -f -o /tmp/strace.log /usr/local/bin/poker-server"
	@echo "Then inspect /tmp/strace.log for syscalls (e.g., open() failures on missing files)"
