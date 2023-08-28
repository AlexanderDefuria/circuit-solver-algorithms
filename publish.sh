#!/bin/bash

jq '.version' ./package.json
cp ./package.json ./pkg/package.json
