#!/bin/bash
echo "Building MiniOS Docker image..."
docker build -t minios . && docker run -it --rm minios
