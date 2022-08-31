import json
import xrpl
from constants import TEST_ACCOUNT, JSON_RPC_URL, GENESIS_SEED
from xrpl.clients import JsonRpcClient
from xrpl.wallet import Wallet

# Creates client to talk to a node.
client = JsonRpcClient(JSON_RPC_URL)

# Creates a Genesis account wallet.
genesis_wallet = Wallet(seed=GENESIS_SEED, sequence=16237283)

# Creates transaction details.
payment_details = xrpl.models.transactions.Payment(
    account=genesis_wallet.classic_address,
    amount=xrpl.utils.xrp_to_drops(5000),
    destination=TEST_ACCOUNT,
)

# Signs the transaction.
signed_tx = xrpl.transaction.safe_sign_and_autofill_transaction(payment_details, genesis_wallet, client)

# Tries to submit the transaction
try:
    tx_response = xrpl.transaction.send_reliable_submission(signed_tx, client)
except xrpl.transaction.XRPLReliableSubmissionException as e:
    exit(f"Submit failed: {e}")

# Prints details about transaction.
print(json.dumps(tx_response.result, indent=4, sort_keys=True))
metadata = tx_response.result.get("meta", {})
if metadata.get("TransactionResult"):
    print("Result code:", metadata["TransactionResult"])
if metadata.get("delivered_amount"):
    print("XRP delivered:", xrpl.utils.drops_to_xrp(
        metadata["delivered_amount"]))
