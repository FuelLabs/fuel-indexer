#!/bin/bash

set -o errexit # abort on nonzero exitstatus
set -o nounset # abort on unbound variable

set -o allexport && source .env && set +o allexport 

if [ "${k8s_provider}" == "eks" ]; then
    echo "Updating your kube context locally ...."
    aws eks update-kubeconfig --name ${TF_VAR_eks_cluster_name}
    if [ "${postgres_enabled}" == "true" ]; then
        cd ../charts/postgres
        mv values.yaml values.template
        envsubst < values.template > values.yaml
        rm values.template
        echo "Deploying postgres helm chart to ${TF_VAR_eks_cluster_name} ...."
        helm upgrade postgres . \
                  --values values.yaml \
                  --install \
                  --create-namespace \
                  --namespace=${k8s_indexer_namespace} \
                  --wait \
                  --timeout 8000s \
                  --debug   
    else
       echo "Postgres deployment is not enabled ... skipping this steping ..."
    fi  
else
   echo "You have inputted a non-supported kubernetes provider in your .env"
fi