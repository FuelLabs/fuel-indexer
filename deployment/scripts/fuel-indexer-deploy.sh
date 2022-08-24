#!/bin/bash

set -o errexit # abort on nonzero exitstatus
set -o nounset # abort on unbound variable

set -o allexport && source .env && set +o allexport 

if [ "${k8s_provider}" == "eks" ]; then
    echo "Updating your kube context locally ...."
    aws eks update-kubeconfig --name ${TF_VAR_eks_cluster_name}
    echo "Copying manifest.yaml into deployment context..."
    cp manifests/${manifest_file} ../charts/manifest.yaml
    cd ../charts
    mv values.yaml values.template
    envsubst < values.template > values.yaml
    rm values.template
    echo "Deploying fuel-indexer helm chart to ${TF_VAR_eks_cluster_name} ...."
    helm upgrade fuel-indexer . \
              --values values.yaml \
              --install \
              --create-namespace \
              --namespace=${k8s_namespace} \
              --wait \
              --timeout 8000s \
              --debug    
else
   echo "You have inputted a non-supported kubernetes provider in your .env"
fi