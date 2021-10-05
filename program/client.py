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

import time

parser = argparse.ArgumentParser(description='Add Questionnaire struct on-chain')
parser.add_argument('--size_A', dest='size_A', help='get size of A', type=float)
parser.add_argument('--size_B', dest='size_B', help='get size of B', type=float)

args = parser.parse_args()


Alice = Keypair()
Bob = Keypair()

temp = Keypair()

Y_A = Keypair()

escrow_MD = Keypair()

Rent = 

token_program =





### OLD IS GOLD
http_client = Client("https://api.devnet.solana.com")
ID = "A4vEBwVMoEZ8j4gthtMnm3MLgKtZoSrw7kYGq4KotE2Q"

newacc = Keypair()


instruction = create_account(CreateAccountParams(from_pubkey=x.public_key, new_account_pubkey=newacc.public_key, lamports=3800000, space=416, program_id=base58.b58decode(ID)))


tix = Transaction().add(instruction)



mystring = bytes(x0[32:])
mystring += args.school.encode('utf-8').ljust(128,b'\x00')
mystring += args.email.encode('utf-8').ljust(128,b'\x00')
link = "https://media.giphy.com/media/QxS7h2qA4tu5dHytSC/giphy.gif"
mystring += link.encode('utf-8').ljust(128,b'\x00')


instruction_data = bytearray(mystring)

accmeta = [AccountMeta(newacc.public_key, True, True)]
tix = tix.add(TransactionInstruction(data=instruction_data, program_id=ID, keys=accmeta))

print(len(instruction_data))

y = http_client.send_transaction(tix, *[x,newacc])
print(y)

time.sleep(10)

print(http_client.get_account_info(newacc.public_key))

print(newacc.public_key)