FROM ubuntu:22.04

# Prevent interactive prompts during apt-get
ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies required by Connect IQ SDK Manager and GUI
RUN apt-get update && apt-get install -y \
    wget \
    unzip \
    openjdk-17-jre \
    libwebkit2gtk-4.0-37 \
    libcanberra-gtk-module \
    libcanberra-gtk3-module \
    libusb-1.0-0 \
    x11-utils \
    libcurl4 \
    && rm -rf /var/lib/apt/lists/*

# Add developer user with UID 1000 so volume permissions match the host
RUN useradd -ms /bin/bash -u 1000 developer

# Download and extract the latest Connect IQ SDK Manager
WORKDIR /opt/ciq
RUN wget -qO sdkmanager.zip "https://developer.garmin.com/downloads/connect-iq/sdk-manager/connectiq-sdk-manager-linux.zip" && \
    unzip sdkmanager.zip && \
    rm sdkmanager.zip && \
    chmod +x bin/sdkmanager

# Create a proxy script for SDK binaries that points to the volume-installed SDK
RUN echo '#!/bin/bash' > /opt/ciq/bin/proxy.sh && \
    echo 'if [ ! -f /home/developer/.Garmin/ConnectIQ/current-sdk.cfg ]; then' >> /opt/ciq/bin/proxy.sh && \
    echo '    echo "Error: No active SDK found. Please run make setup-device and download an SDK first." >&2' >> /opt/ciq/bin/proxy.sh && \
    echo '    exit 1' >> /opt/ciq/bin/proxy.sh && \
    echo 'fi' >> /opt/ciq/bin/proxy.sh && \
    echo 'SDK_PATH=$(cat /home/developer/.Garmin/ConnectIQ/current-sdk.cfg)' >> /opt/ciq/bin/proxy.sh && \
    echo 'CMD=$(basename "$0")' >> /opt/ciq/bin/proxy.sh && \
    echo 'exec "${SDK_PATH}/bin/${CMD}" "$@"' >> /opt/ciq/bin/proxy.sh && \
    chmod +x /opt/ciq/bin/proxy.sh && \
    ln -s /opt/ciq/bin/proxy.sh /opt/ciq/bin/monkeyc && \
    ln -s /opt/ciq/bin/proxy.sh /opt/ciq/bin/monkeydo && \
    ln -s /opt/ciq/bin/proxy.sh /opt/ciq/bin/connectiq && \
    ln -s /opt/ciq/bin/proxy.sh /opt/ciq/bin/simulator

USER developer
ENV PATH="/opt/ciq/bin:${PATH}"
