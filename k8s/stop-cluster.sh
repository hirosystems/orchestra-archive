#!/bin/bash

# Shutdown the bitcoin-node
kubectl delete configmap bitcoin-node-config 
kubectl delete statefulset bitcoin-node
kubectl delete service bitcoin-node

# Shutdown the stacks-node
# kubectl delete configmap stacks-config 
# kubectl delete statefulset stacks
# kubectl delete service stacks

