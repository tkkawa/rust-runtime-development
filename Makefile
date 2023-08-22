PREFIX ?= /usr/local
BINDIR := $(PREFIX)/sbin
PROJECT_NAME=rust-runtime
IMAGE_NAME=${USER}_${PROJECT_NAME}
CONTAINER_NAME=${USER}_${PROJECT_NAME}
SHM_SIZE=2g
FORCE_RM=true

build_runtime:
	docker build \
		-f docker/Dockerfile \
		-t $(IMAGE_NAME)-build \
		--no-cache \
		--force-rm=$(FORCE_RM) \
		.

run_runtime:
	docker run \
		-dit \
		-v $(PWD):/workspace/crate \
		--name $(CONTAINER_NAME)-build \
		--rm \
		--cap-add ALL \
		--shm-size $(SHM_SIZE) \
		--privileged \
		$(IMAGE_NAME)-build \
		/bin/bash

exec_runtime:
	docker exec \
		-it \
		$(CONTAINER_NAME)-build /bin/bash

stop_runtime:
	docker stop $(CONTAINER_NAME)-build

build_test:
	docker build \
		-f docker/test/Dockerfile \
		-t $(IMAGE_NAME)-test \
		--no-cache \
		--force-rm=$(FORCE_RM) \
		.

run_test:
	docker run \
		-dit \
		-v $(PWD):/workspace/crate \
		--name $(CONTAINER_NAME)-test \
		--rm \
		--cap-add ALL \
		--shm-size $(SHM_SIZE) \
		--privileged \
		$(IMAGE_NAME)-test \
		/bin/bash

exec_test:
	docker exec \
		-it \
		$(CONTAINER_NAME)-test /bin/bash

stop_test:
	docker stop $(CONTAINER_NAME)-test

install:
	install -D -m0755 /workspace/crate/target/debug/rust-runtime $(BINDIR)/rust-runtime
