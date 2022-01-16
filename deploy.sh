#! /bin/bash

set -e

echo "Building..."
docker build -t danbruder/twhn_api:latest .

echo "Pushing..."
docker push danbruder/twhn_api:latest

echo "Deploying..."
caprover deploy -i danbruder/twhn_api:latest -a twhn -n captain-01
