IMAGE_NAME = hetzner.camel-yo.ts.net:5000/patwoz-notify
CONTAINER_NAME = patwoz-notify

docker-image:
	docker build . -t $(IMAGE_NAME)

docker-run:
	docker run -p 3000:3000 -v ${PWD}:/app -t $(IMAGE_NAME):latest

docker-stop:
	docker stop $(CONTAINER_NAME)

docker-kill:
	docker kill $(CONTAINER_NAME)

docker-push: image
	docker push $(IMAGE_NAME)

docker-deploy: push
	ssh hetzner "docker pull $(IMAGE_NAME)"
	ssh hetzner "docker run -v /var/run/docker.sock:/var/run/docker.sock containrrr/watchtower --label-enable --run-once"

