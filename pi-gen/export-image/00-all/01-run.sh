#!/bin/bash -e
# pi-gen/export-image/00-all/01-run.sh — Common image export setup

IMG_FILE="${STAGE_WORK_DIR}/image-${IMG_NAME}-${IMG_SUFFIX}"

# Ensure deploy directory exists
mkdir -p "${DEPLOY_DIR}"