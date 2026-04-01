APP_NAME=LiveSegments
DEVICE=edge840
SDK_IMAGE=garmin-sdk-local

.PHONY: build keys setup simulate run

setup:
	mkdir -p bin source resources
	chmod 777 bin
	docker build -t $(SDK_IMAGE) -f Dockerfile .

setup-device:
	docker volume create garmin-sdk-devices
	docker run --rm --user root -v garmin-sdk-devices:/home/developer/.Garmin $(SDK_IMAGE) chown -R 1000:1000 /home/developer/.Garmin
	xhost +local:root
	docker run -it --rm \
		-e DISPLAY \
		-v /tmp/.X11-unix:/tmp/.X11-unix \
		-v garmin-sdk-devices:/home/developer/.Garmin \
		$(SDK_IMAGE) /opt/ciq/bin/sdkmanager

build:
	docker run --rm -v $(PWD):/app -w /app \
		-v garmin-sdk-devices:/home/developer/.Garmin \
		$(SDK_IMAGE) monkeyc -o bin/$(APP_NAME).prg -f monkey.jungle -d $(DEVICE) -y developer_key.der -l 0
keys:
	docker run --rm -v $(PWD):/app -w /app alpine/openssl genrsa -out developer_key.pem 4096
	docker run --rm -v $(PWD):/app -w /app alpine/openssl pkcs8 -topk8 -inform PEM -outform DER -in developer_key.pem -out developer_key.der -nocrypt
	docker run --rm -v $(PWD):/app alpine/openssl sh -c "chmod 666 /app/developer_key.*"

simulate:
	xhost +local:root
	docker run -d --rm --net host \
		-e DISPLAY \
		-v /tmp/.X11-unix:/tmp/.X11-unix \
		-v garmin-sdk-devices:/home/developer/.Garmin \
		$(SDK_IMAGE) simulator

run: build
	docker run --rm --net host -v $(PWD):/app -w /app \
		-v garmin-sdk-devices:/home/developer/.Garmin \
		$(SDK_IMAGE) monkeydo bin/$(APP_NAME).prg $(DEVICE)
