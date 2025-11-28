#!/bin/bash

dir="$( dirname "$0" )"
docker build -o "platform/built/$(basename "$dir")/" -f "$dir/Dockerfile" .
