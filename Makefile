IMAGE_NAME = hetzner.camel-yo.ts.net:5000/patwoz-notify
CONTAINER_NAME = patwoz-notify

db-create:
	mkdir -p data
	sqlx database create
	sqlx migrate run

init: db-create

docker-image:
	docker build --build-arg TARGET=aarch64-unknown-linux-gnu . -t $(IMAGE_NAME)

docker-run:
	docker run --rm -p 3000:3000 \
		--name ${CONTAINER_NAME}\
		-v ${PWD}/.env:/app/.env\
		-v ${PWD}/data:/app/data\
		-t $(IMAGE_NAME):latest

docker-stop:
	docker stop $(CONTAINER_NAME)

docker-kill:
	docker kill $(CONTAINER_NAME)

docker-push: docker-image
	docker push $(IMAGE_NAME)

docker-inspect:
	docker inspect $(IMAGE_NAME)

docker-dive:
	dive $(IMAGE_NAME)

docker-deploy: docker-push
	ssh hetzner "docker pull $(IMAGE_NAME)"
	ssh hetzner "docker run -v /var/run/docker.sock:/var/run/docker.sock containrrr/watchtower --label-enable --run-once"

