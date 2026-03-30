APP_NAME=LiveSegments
DEVICE=edge1040
SDK_IMAGE=waterkip/connectiq-sdk-docker:latest

.PHONY: build keys setup

setup:
	mkdir -p bin source resources

build:
	docker run --rm -v $(PWD):/app -w /app $(SDK_IMAGE) monkeyc -o bin/$(APP_NAME).prg -f monkey.jungle -d $(DEVICE) -y developer_key.der

keys:
	docker run --rm -v $(PWD):/app -w /app alpine/openssl genrsa -out developer_key.pem 4096
	docker run --rm -v $(PWD):/app -w /app alpine/openssl pkcs8 -topk8 -inform PEM -outform DER -in developer_key.pem -out developer_key.der -nocrypt
