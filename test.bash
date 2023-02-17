#!/bin/bash
#
#
foo=$1

if [[ ${foo} =~ ^refs/heads/v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "is_release_branch=true"
fi
