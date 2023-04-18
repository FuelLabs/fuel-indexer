#!/bin/bash

set -o errexit # abort on nonzero exitstatus
set -o nounset # abort on unbound variable

set -o allexport && source .env && set +o allexport 

if [ "${k8s_provider}" == "eks" ]; then
    echo "Updating your kube context locally ...."
    aws eks update-kubeconfig --name ${TF_VAR_eks_cluster_name}
    if [ "${postgres_enabled}" == "true" ]; then
        echo "Deleting postgres helm chart on ${TF_VAR_eks_cluster_name} ...."
        helm delete postgres \
                  --namespace ${k8s_namespace} \
                  --wait \
                  --timeout 8000s \
                  --debug
    else
       echo "Postgres deployment is not enabled ... skipping this steping ..." 
    fi 
else
   echo "You have inputted a non-supported kubernetes provider in your .env"
fi
