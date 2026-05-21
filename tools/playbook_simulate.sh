#!/bin/bash
# tools/playbook_simulate.sh
# Static simulation of a playbook - prints the sequence of guard calls

set -euo pipefail

PLAYBOOK="$1"
if [ -z "$PLAYBOOK" ]; then
    echo "Usage: $0 <playbook.aln>"
    exit 1
fi

if [ ! -f "$PLAYBOOK" ]; then
    echo "ERROR: Playbook file not found: $PLAYBOOK"
    exit 1
fi

echo "=== Simulating playbook: $PLAYBOOK ==="
echo ""

# Extract and display steps, intents, guards, decisions, and actuation hints
grep -E 'step|intent|guard|require_decision|actuation_hint' "$PLAYBOOK" | while read -r line; do
    echo "$line"
done

echo ""
echo "=== End of simulation ==="
