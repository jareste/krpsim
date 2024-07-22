IMAGE_NAME = rust-app
CONTAINER_NAME = rust-container

all: build run

build:
	docker build -t $(IMAGE_NAME) .

run:
	docker run --name $(CONTAINER_NAME) --rm $(IMAGE_NAME)

down:
	docker stop $(CONTAINER_NAME) || true

clean:
	docker rmi $(IMAGE_NAME) -f
	docker rm $(CONTAINER_NAME) || true

.PHONY: all build run down clean