#!/usr/bin/env bash
set -euo pipefail

# PulsarTrack - Contract Initialization Script
# Runs after deploy.sh to initialize all contracts with admin and config

NETWORK="${STELLAR_NETWORK:-testnet}"
IDENTITY="${STELLAR_IDENTITY:-pulsartrack-deployer}"
DEPLOY_FILE=""
TOKEN_ADDRESS=""

# Simple argument parsing
while [[ $# -gt 0 ]]; do
  case $1 in
    --network)
      NETWORK="$2"
      shift 2
      ;;
    --identity)
      IDENTITY="$2"
      shift 2
      ;;
    --token)
      TOKEN_ADDRESS="$2"
      shift 2
      ;;
    --file)
      DEPLOY_FILE="$2"
      shift 2
      ;;
    *)
      # Legacy support for passing deploy file as first positional arg
      if [ -z "$DEPLOY_FILE" ]; then
        DEPLOY_FILE="$1"
      fi
      shift
      ;;
  esac
done

# Default deployment file if not provided
if [ -z "$DEPLOY_FILE" ]; then
  DEPLOY_FILE="$(dirname "$0")/../deployments/deployed-$NETWORK.json"
fi

if [ ! -f "$DEPLOY_FILE" ]; then
  echo "Error: Deployment file not found: $DEPLOY_FILE"
  echo "Run deploy.sh first."
  exit 1
fi

# Token derivation logic
if [ -z "$TOKEN_ADDRESS" ]; then
  if [ "$NETWORK" = "testnet" ]; then
    # Testnet XLM Contract ID
    TOKEN_ADDRESS="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
    echo "[Info] Using Testnet XLM: $TOKEN_ADDRESS"
  elif [ "$NETWORK" = "mainnet" ]; then
    # Mainnet XLM Contract ID
    TOKEN_ADDRESS="CAS3J7GYCCX3TP37S65CC3S6SRE6IDYRRW3B6LUL2B6GND5X67U5TBBT"
    echo "[Info] Using Mainnet XLM: $TOKEN_ADDRESS"
  else
    echo "Error: No token address provided and unknown network: $NETWORK"
    echo "Use --token <address> to specify the payment token."
    exit 1
  fi
fi

echo "=============================================="
echo "  PulsarTrack - Contract Initialization"
echo "  Network: $NETWORK"
echo "  Deployment: $DEPLOY_FILE"
echo "  Token: $TOKEN_ADDRESS"
echo "=============================================="

ADMIN_ADDRESS=$(stellar keys address "$IDENTITY")
ORACLE_ADDRESS="${STELLAR_ORACLE_ADDRESS:-$ADMIN_ADDRESS}"
TREASURY_ADDRESS="${STELLAR_TREASURY_ADDRESS:-$ADMIN_ADDRESS}"
PLATFORM_ADDRESS="${STELLAR_PLATFORM_ADDRESS:-$ADMIN_ADDRESS}"
RELAYER_ADDRESS="${STELLAR_RELAYER_ADDRESS:-$ADMIN_ADDRESS}"
MULTISIG_SIGNERS="${STELLAR_MULTISIG_SIGNERS:-[\"$ADMIN_ADDRESS\"]}"
MULTISIG_REQUIRED="${STELLAR_MULTISIG_REQUIRED:-1}"
VOTING_PERIOD="${STELLAR_GOV_VOTING_PERIOD:-3600}"
QUORUM_BPS="${STELLAR_GOV_QUORUM_BPS:-1000}"
PASS_THRESHOLD="${STELLAR_GOV_PASS_THRESHOLD:-51}"
PROPOSER_MIN="${STELLAR_GOV_PROPOSER_MIN:-100}"
MIN_DELAY_SECS="${STELLAR_TIMELOCK_MIN_DELAY_SECS:-3600}"
MAX_DELAY_SECS="${STELLAR_TIMELOCK_MAX_DELAY_SECS:-86400}"
FILING_FEE="${STELLAR_DISPUTE_FILING_FEE:-1000}"
MIN_ATTESTERS="${STELLAR_PERFORMANCE_MIN_ATTESTERS:-2}"

MISSING=0
INITIALIZED=0

# Helper: call a contract function
call_contract() {
  local CONTRACT_ID="$1"
  local FUNCTION="$2"
  shift 2

  echo "[Invoke] $FUNCTION on $CONTRACT_ID..."
  local OUTPUT
  if ! OUTPUT=$(stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    -- "$FUNCTION" "$@" 2>&1); then
    if [[ "$OUTPUT" == *"already initialized"* ]]; then
      echo "  Already initialized"
    else
      echo "$OUTPUT" >&2
      return 1
    fi
  fi
  INITIALIZED=$((INITIALIZED + 1))
  echo "  OK"
}

read_contract() {
  python3 - "$DEPLOY_FILE" "$1" <<'PY' 2>/dev/null
import json
import sys

deploy_file, name = sys.argv[1], sys.argv[2]
with open(deploy_file, encoding="utf-8") as f:
    data = json.load(f)
print(data["contracts"].get(name, ""))
PY
}

initialize_contract() {
  local NAME="$1"
  shift
  local CONTRACT_ID
  CONTRACT_ID=$(read_contract "$NAME")
  if [ -z "$CONTRACT_ID" ] || [ "$CONTRACT_ID" = "null" ]; then
    echo "[Error] Missing contract ID for $NAME in $DEPLOY_FILE" >&2
    MISSING=$((MISSING + 1))
    return 0
  fi
  echo "[Init] Initializing $NAME..."
  call_contract "$CONTRACT_ID" initialize "$@"
}

GOV_TOKEN_ID=$(read_contract governance_token)
CAMPAIGN_ORCHESTRATOR_ID=$(read_contract campaign_orchestrator)
CAMPAIGN_LIFECYCLE_ID=$(read_contract campaign_lifecycle)
if [ -z "$GOV_TOKEN_ID" ] || [ -z "$CAMPAIGN_ORCHESTRATOR_ID" ] || [ -z "$CAMPAIGN_LIFECYCLE_ID" ]; then
  echo "[Error] Deployment file is missing governance_token, campaign_orchestrator, or campaign_lifecycle" >&2
  exit 1
fi

initialize_contract ad_registry --admin "$ADMIN_ADDRESS"
initialize_contract campaign_orchestrator --admin "$ADMIN_ADDRESS" --token_address "$TOKEN_ADDRESS"
initialize_contract escrow_vault --admin "$ADMIN_ADDRESS" --token_address "$TOKEN_ADDRESS" --oracle "$ORACLE_ADDRESS"
initialize_contract fraud_prevention --admin "$ADMIN_ADDRESS"
initialize_contract payment_processor --admin "$ADMIN_ADDRESS" --treasury "$TREASURY_ADDRESS"

initialize_contract governance_token --admin "$ADMIN_ADDRESS"
initialize_contract governance_dao \
  --admin "$ADMIN_ADDRESS" \
  --governance_token "$GOV_TOKEN_ID" \
  --voting_period "$VOTING_PERIOD" \
  --quorum_bps "$QUORUM_BPS" \
  --pass_threshold "$PASS_THRESHOLD" \
  --proposer_min "$PROPOSER_MIN"
initialize_contract governance_core --admin "$ADMIN_ADDRESS"
initialize_contract timelock_executor \
  --admin "$ADMIN_ADDRESS" \
  --executor "$ADMIN_ADDRESS" \
  --min_delay_secs "$MIN_DELAY_SECS" \
  --max_delay_secs "$MAX_DELAY_SECS"

initialize_contract publisher_verification --admin "$ADMIN_ADDRESS" --orchestrator "$CAMPAIGN_ORCHESTRATOR_ID"
initialize_contract publisher_network --admin "$ADMIN_ADDRESS"
initialize_contract publisher_reputation --admin "$ADMIN_ADDRESS" --oracle "$ORACLE_ADDRESS"

initialize_contract analytics_aggregator --admin "$ADMIN_ADDRESS" --oracle "$ORACLE_ADDRESS"
initialize_contract campaign_analytics --admin "$ADMIN_ADDRESS" --oracle "$ORACLE_ADDRESS"
initialize_contract campaign_lifecycle --admin "$ADMIN_ADDRESS"

initialize_contract privacy_layer --admin "$ADMIN_ADDRESS"
initialize_contract targeting_engine --admin "$ADMIN_ADDRESS" --lifecycle_contract "$CAMPAIGN_LIFECYCLE_ID"
initialize_contract audience_segments --admin "$ADMIN_ADDRESS"
initialize_contract identity_registry --admin "$ADMIN_ADDRESS"
initialize_contract kyc_registry --admin "$ADMIN_ADDRESS"

initialize_contract auction_engine --admin "$ADMIN_ADDRESS" --token "$TOKEN_ADDRESS"
initialize_contract creative_marketplace --admin "$ADMIN_ADDRESS" --token "$TOKEN_ADDRESS"

initialize_contract subscription_manager \
  --admin "$ADMIN_ADDRESS" \
  --token "$TOKEN_ADDRESS" \
  --treasury "$TREASURY_ADDRESS" \
  --orchestrator "$CAMPAIGN_ORCHESTRATOR_ID"
initialize_contract subscription_benefits --admin "$ADMIN_ADDRESS"
initialize_contract liquidity_pool --admin "$ADMIN_ADDRESS" --token "$TOKEN_ADDRESS"
initialize_contract milestone_tracker --admin "$ADMIN_ADDRESS" --oracle "$ORACLE_ADDRESS"
initialize_contract multisig_treasury --admin "$ADMIN_ADDRESS" --initial_signers "$MULTISIG_SIGNERS" --required "$MULTISIG_REQUIRED"
initialize_contract oracle_integration --admin "$ADMIN_ADDRESS"
initialize_contract payout_automation --admin "$ADMIN_ADDRESS" --token "$TOKEN_ADDRESS"
initialize_contract performance_oracle --admin "$ADMIN_ADDRESS" --min_attesters "$MIN_ATTESTERS"
initialize_contract recurring_payment --admin "$ADMIN_ADDRESS"
initialize_contract refund_processor --admin "$ADMIN_ADDRESS" --token "$TOKEN_ADDRESS"
initialize_contract revenue_settlement \
  --admin "$ADMIN_ADDRESS" \
  --token "$TOKEN_ADDRESS" \
  --treasury "$TREASURY_ADDRESS" \
  --platform "$PLATFORM_ADDRESS"
initialize_contract rewards_distributor --admin "$ADMIN_ADDRESS" --reward_token "$TOKEN_ADDRESS"

initialize_contract token_bridge --admin "$ADMIN_ADDRESS" --relayer "$RELAYER_ADDRESS"
initialize_contract wrapped_token --admin "$ADMIN_ADDRESS" --relayer "$RELAYER_ADDRESS"
initialize_contract dispute_resolution --admin "$ADMIN_ADDRESS" --token "$TOKEN_ADDRESS" --filing_fee "$FILING_FEE"
initialize_contract budget_optimizer --admin "$ADMIN_ADDRESS" --oracle "$ORACLE_ADDRESS"
initialize_contract anomaly_detector --admin "$ADMIN_ADDRESS" --oracle "$ORACLE_ADDRESS"

initialize_contract access_control --admin "$ADMIN_ADDRESS"
initialize_contract treasury_manager --admin "$ADMIN_ADDRESS" --token "$TOKEN_ADDRESS"
initialize_contract vesting_schedule --admin "$ADMIN_ADDRESS"
initialize_contract whitelist_registry --admin "$ADMIN_ADDRESS"

echo ""
if [ "$MISSING" -gt 0 ]; then
  echo "[Error] $MISSING contract(s) missing from deployment file" >&2
  exit 1
fi
echo "=============================================="
echo "  Initialization complete! ($INITIALIZED contracts)"
echo "=============================================="
