#!/bin/bash

# Configure and boot a bitcoin-node
kubectl apply -f bitcoin-node/configmap.yaml
kubectl apply -f bitcoin-node/statefulset.yaml
kubectl apply -f bitcoin-node/service.yaml

# Configure and boot a stacks-node
kubectl apply -f stacks-node/configmap.yaml
kubectl apply -f stacks-node/statefulset.yaml
kubectl apply -f stacks-node/service.yaml
