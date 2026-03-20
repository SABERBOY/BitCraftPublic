#!/bin/bash

read -p "Enter the spacetime server name (e.g. bitcraft-staging): " host

#spacetime call -s "$host" "bitcraft-live-global" admin_update_empire_ranks

for i in {1..25}; do
  #spacetime call -s "$host" "bitcraft-live-$i" migrate_character_stats
done
