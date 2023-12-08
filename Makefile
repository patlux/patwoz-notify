docker-build:
	docker build -t patwoz-notify:latest .

docker-run:
	docker run -p 1337:1337 -v $(pwd):/app -t patwoz-notify:latest
