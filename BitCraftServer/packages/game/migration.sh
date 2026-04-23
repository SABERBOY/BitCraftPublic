#!/bin/bash

read -p "Enter the spacetime server name (e.g. bitcraft-staging): " host

for i in {1..25}; do
  spacetime call -s "$host" "bitcraft-live-$i" #add reducer here
done
