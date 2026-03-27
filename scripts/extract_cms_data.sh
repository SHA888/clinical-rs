#!/bin/bash
# Extract CMS ICD-10-CM code tables

cd /home/kd/clinical-rs/crates/medcodes/data

echo "Extracting CMS ICD-10-CM code tables..."
unzip -q april-1-2026-code-tables-tabular-and-index.zip

echo "Listing extracted files:"
ls -lah

echo ""
echo "First few lines of each file:"
for file in *.txt; do
    echo "=== $file ==="
    head -5 "$file"
    echo ""
done
