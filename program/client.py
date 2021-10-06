import solana
from solana.rpc.api import Client
from solana.keypair import Keypair
from solana.transaction import TransactionInstruction, Transaction, AccountMeta
import argparse
from solana.publickey import PublicKey
from solana.blockhash import Blockhash
from solana.system_program import *
import base58
from solana.sysvar import SYSVAR_RENT_PUBKEY
from solana.system_program import SYS_PROGRAM_ID
from spl.token.constants import TOKEN_PROGRAM_ID
from spl.token.client import Token

import time

client = Client("https://api.devnet.solana.com")
ID = "A4vEBwVMoEZ8j4gthtMnm3MLgKtZoSrw7kYGq4KotE2Q"#"A4vEBwVMoEZ8j4gthtMnm3MLgKtZoSrw7kYGq4KotE2Q"
program_id=base58.b58decode(ID)

parser = argparse.ArgumentParser(description='Add Questionnaire struct on-chain')
parser.add_argument('--size_A', dest='size_A', help='get size of A', type=float)
parser.add_argument('--size_B', dest='size_B', help='get size of B', type=float)
args = parser.parse_args()

God = Keypair()
client.request_airdrop(God.public_key, 1000000000000)



Alice = Keypair()
Bob = Keypair()

# # create these 6 owned
# X_A = Keypair()
# Y_A = Keypair()
# instruction = create_account(CreateAccountParams(from_pubkey=Alice.public_key, new_account_pubkey=X_vault_pk.public_key, lamports=3800000, space=416, program_id=base58.b58decode(ID)))


# X_B = Keypair()
# Y_B = Keypair()


X_mint_kp = Keypair()
Y_mint_kp = Keypair()

X_token = Token(client, X_mint_kp.public_key, TOKEN_PROGRAM_ID, God)
Y_token = Token(client, Y_mint_kp.public_key, TOKEN_PROGRAM_ID, God)
X_token.create_mint(client, God, God.public_key, 9, TOKEN_PROGRAM_ID)
Y_token.create_mint(client, God, God.public_key, 9, TOKEN_PROGRAM_ID)

token_program_pk = TOKEN_PROGRAM_ID



X_vault_pk, _ = PublicKey.find_program_address(["x_vault"], program_id) ## TODO: make byte
# instruction = create_account(CreateAccountParams(from_pubkey=Alice.public_key, new_account_pubkey=X_vault_pk.public_key, lamports=3800000, space=416, program_id=base58.b58decode(ID)))

Y_vault_pk, _ = PublicKey.find_program_address(["y_vault"], program_id)
# instruction = create_account(CreateAccountParams(from_pubkey=Alice.public_key, new_account_pubkey=Y_vault_pk.public_key, lamports=3800000, space=416, program_id=base58.b58decode(ID)))

escrow_pk, _ = PublicKey.find_program_address(["escrow"], program_id)
# instruction = create_account(CreateAccountParams(from_pubkey=Alice.public_key, new_account_pubkey=escrow_MD_pk.public_key, lamports=3800000, space=416, program_id=base58.b58decode(ID)))







# Account metas
account_metas = [
(alice.public_key, False, False),
(bob.public_key, False, False),
(X_mint.pubkey, False, False),
(Y_mint.pubkey, False, False),
(X_vault_pk, False, True),
(Y_vault_pk, False, True),
(escrow_pk, False, False),
(TOKEN_PROGRAM_ID, False, False),
(SYS_PROGRAM_ID, False, False),
(SYSVAR_RENT_PUBKEY, False, False),
]


idx = 0

instruction_data = idx.to_bytes(1,"little") + args.size_A.to_bytes(8,"little") + args.size_B.to_bytes(8,"little")




# transactions
instruction = TransactionInstruction(data=instruction_data,program_id=ID,keys=account_metas)

tix = Transaction().add(instruction)




print(len(instruction_data))

y = client.send_transaction(tix, [])
print(y)

time.sleep(10)

print("ESCROW")
print(client.get_account_info(escrow_pk))
print("X VAULT")
print(client.get_account_info(X_vault_pk))
