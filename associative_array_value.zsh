#!/bin/zsh

local -A a={{a}}

for key val in ${(@kv)a}; do
  echo "$key"
  echo "$val"
done

